use crate::{inspect::print_network_message, structs::*};
use async_std::io;
use async_std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use async_std::sync::{channel, Arc, Mutex, Receiver, Sender};
use async_trait::async_trait;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use futures_codec::{Framed, LengthCodec};
use std::future::Future;

#[derive(Debug)]
pub enum ControlMessage {
    /// Stop the server.
    Stop,
}

#[derive(Debug)]
pub enum AppMessage {
    /// Send a message to the DirectPlay application.
    Send(u32, u32, Vec<u8>),
}

/// Trait for custom Service Provider implementations.
#[async_trait]
pub trait ServiceProvider: Sync + Send {
    async fn enum_sessions(
        &mut self,
        controller: AppController,
        id: u32,
        data: EnumSessionsData,
    ) -> io::Result<()>;
    async fn open(&mut self, controller: AppController, id: u32, data: OpenData) -> io::Result<()>;
    async fn create_player(
        &mut self,
        controller: AppController,
        id: u32,
        data: CreatePlayerData,
    ) -> io::Result<()>;
    async fn reply(
        &mut self,
        controller: AppController,
        id: u32,
        data: ReplyData,
    ) -> io::Result<()>;
    async fn send(&mut self, controller: AppController, id: u32, data: SendData) -> io::Result<()>;
}

/// Struct containing methods to control the service provider host server.
#[derive(Clone)]
pub struct ServerController {
    sender: Sender<ControlMessage>,
}

impl ServerController {
    /// Create a host server controller.
    ///
    /// Returns the controller and the message stream it generates.
    pub fn create() -> (Self, Receiver<ControlMessage>) {
        // TODO figure out appropriate buffer size
        // May only need to be oneshot
        let (sender, receiver) = channel(5);
        let controller = ServerController { sender };

        (controller, receiver)
    }

    /// Stop the host server.
    ///
    /// Returns a Future, so make sure to consume it.
    pub async fn stop(&mut self) {
        self.sender.send(ControlMessage::Stop).await;
    }
}

/// Controller for sending messages to the game.
#[derive(Clone)]
pub struct AppController {
    sender: Sender<AppMessage>,
    next_message_id: u32,
}

impl AppController {
    pub fn create() -> (Self, Receiver<AppMessage>) {
        // TODO figure out appropriate buffer size
        let (sender, receiver) = channel(5);
        let controller = AppController {
            sender,
            next_message_id: 0,
        };

        (controller, receiver)
    }

    pub async fn send(&mut self, data: Vec<u8>) {
        let msg_id = self.next_message_id;
        self.next_message_id += 1;
        log::debug!("[AppController::send] {}", msg_id);
        self.sender
            .send(AppMessage::Send(msg_id as u32, std::u32::MAX, data))
            .await;
    }

    pub async fn reply(&mut self, id: u32, data: Vec<u8>) {
        let msg_id = self.next_message_id;
        self.next_message_id += 1;
        self.sender
            .send(AppMessage::Send(msg_id as u32, id, data))
            .await;
    }
}

async fn handle_message(
    service_provider: Arc<Mutex<Box<dyn ServiceProvider>>>,
    controller: &mut AppController,
    id: u32,
    method: &[u8],
    message: &[u8],
) -> io::Result<()> {
    match method {
        b"enum" => {
            let enum_sessions = EnumSessionsData {
                message: message.to_vec(),
            };
            print_network_message(message);
            service_provider
                .lock()
                .await
                .enum_sessions(controller.clone(), id, enum_sessions)
                .await
        }
        b"open" => {
            let open = OpenData::parse(message);
            service_provider
                .lock()
                .await
                .open(controller.clone(), id, open)
                .await
        }
        b"crpl" => {
            let create_player = CreatePlayerData::parse(message);
            service_provider
                .lock()
                .await
                .create_player(controller.clone(), id, create_player)
                .await
        }
        b"repl" => {
            let reply = ReplyData::parse(message);
            print_network_message(message);
            service_provider
                .lock()
                .await
                .reply(controller.clone(), id, reply)
                .await
        }
        b"send" => {
            let send = SendData::parse(message);
            print_network_message(message);
            service_provider
                .lock()
                .await
                .send(controller.clone(), id, send)
                .await
        }
        method => {
            log::debug!(
                "[HostServer::process_message] HostServer message: {} {:?}, {:?}",
                id,
                method,
                message
            );
            Ok(())
        }
    }
}

fn handle_connection(
    service_provider: Arc<Mutex<Box<dyn ServiceProvider>>>,
    sock: TcpStream,
) -> io::Result<()> {
    sock.set_nodelay(true)?;
    let (mut writer, mut reader) = Framed::new(sock, LengthCodec).split();
    let (mut app_controller, mut app_receiver) = AppController::create();
    log::debug!("[handle_connection] Connection incoming");

    let read_future = async move {
        while let Some(message) = reader.next().await {
            let mut message = match message {
                Ok(message) if message.len() > 12 => message,
                Ok(message) => {
                    log::warn!(
                        "[handle_connection] invalid message, too short: {:?}",
                        message
                    );
                    continue;
                }
                Err(err) => {
                    log::warn!("[handle_connection] Request error: {:?}", err);
                    continue;
                }
            };
            let id = {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&message.split_to(4));
                u32::from_be_bytes(bytes)
            };
            let _reply_id = {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&message.split_to(4));
                u32::from_be_bytes(bytes)
            };
            let method = message.split_to(4);
            handle_message(
                Arc::clone(&service_provider),
                &mut app_controller,
                id,
                &method,
                &message,
            )
            .await
            .unwrap();
        }
        log::debug!("[handle_connection] Connection finished");
    };

    let write_future = async move {
        while let Some(app_message) = app_receiver.next().await {
            match app_message {
                AppMessage::Send(msg_id, reply_to_id, data) => {
                    log::debug!(
                        "[handle_connection] Send message {} in reply to {}",
                        msg_id,
                        reply_to_id
                    );
                    let mut message = vec![0; data.len() + 12];
                    (&mut message[0..4]).copy_from_slice(&msg_id.to_be_bytes());
                    (&mut message[4..8]).copy_from_slice(&reply_to_id.to_be_bytes());
                    (&mut message[8..12]).copy_from_slice(&0u32.to_be_bytes());
                    (&mut message[12..]).copy_from_slice(&data);
                    writer.send(message.into()).await.unwrap();
                }
            }
        }
    };

    async_std::task::spawn(read_future);
    async_std::task::spawn(write_future);
    Ok(())
}

#[derive(Debug)]
enum EventType {
    Socket(TcpStream),
    Control(ControlMessage),
}

pub struct HostServer {
    address: SocketAddr,
    controller: ServerController,
    receiver: Receiver<ControlMessage>,
    service_provider: Box<dyn ServiceProvider>,
}

impl HostServer {
    pub fn new(port: u16, service_provider: Box<dyn ServiceProvider>) -> Self {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let (controller, receiver) = ServerController::create();

        HostServer {
            address,
            controller,
            receiver,
            service_provider,
        }
    }

    pub async fn start(self) -> io::Result<(impl Future<Output = ()>, ServerController)> {
        log::debug!(
            "[HostServer::start] Starting HostServer on {:?}",
            self.address
        );
        let client = TcpListener::bind(&self.address).await?;

        let service_provider = Arc::new(Mutex::new(self.service_provider));
        let _server_controller = self.controller.clone();
        let receiver = self.receiver;
        let server = async move {
            let control_messages = receiver.map(EventType::Control).map(io::Result::Ok);
            let socket_messages = client
                .incoming()
                .map(|result| result.map(EventType::Socket));

            let mut stream = futures::stream::select(socket_messages, control_messages);
            while let Some(message) = stream.next().await {
                log::debug!("[HostServer::start] Receiving message: {:?}", message);
                let message = match message {
                    Err(_) => break,
                    Ok(EventType::Control(ControlMessage::Stop)) => break,
                    Ok(message) => message,
                };

                if let EventType::Socket(socket) = message {
                    log::debug!("[HostServer::start] Spawning socket handler...");
                    handle_connection(Arc::clone(&service_provider), socket).unwrap();
                }
            }
        };

        Ok((server, self.controller))
    }
}
