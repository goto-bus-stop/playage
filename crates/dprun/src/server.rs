use std::mem;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::sync::{Arc, Mutex};
use futures::sync::mpsc::{channel, Sender, Receiver, SendError};
use tokio::io::Result;
use tokio::codec::{Framed, LengthDelimitedCodec};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use bytes::{Bytes, BytesMut, BufMut, ByteOrder, BigEndian};
use crate::structs::*;
use crate::inspect::print_network_message;

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

pub struct SPFuture {
    inner: Box<dyn Future<Item = (), Error = std::io::Error> + Send>,
}
impl SPFuture {
    pub fn new(inner: Box<dyn Future<Item = (), Error = std::io::Error> + Send>) -> Self {
        Self { inner }
    }
}
impl Future for SPFuture {
    type Item = ();
    type Error = std::io::Error;

    fn poll(&mut self) -> std::result::Result<Async<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}

/// Trait for custom Service Provider implementations.
pub trait ServiceProvider: Sync + Send {
    fn enum_sessions(&mut self, controller: AppController, id: u32, data: EnumSessionsData) -> SPFuture;
    fn open(&mut self, controller: AppController, id: u32, data: OpenData) -> SPFuture;
    fn create_player(&mut self, controller: AppController, id: u32, data: CreatePlayerData) -> SPFuture;
    fn reply(&mut self, controller: AppController, id: u32, data: ReplyData) -> SPFuture;
    fn send(&mut self, controller: AppController, id: u32, data: SendData) -> SPFuture;
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
        let controller = ServerController {
            sender,
        };

        (controller, receiver)
    }

    /// Stop the host server.
    ///
    /// Returns a Future, so make sure to consume it.
    pub fn stop(&mut self) -> futures::StartSend<ControlMessage, SendError<ControlMessage>> {
        self.sender.start_send(ControlMessage::Stop)
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
            sender: sender,
            next_message_id: 0,
        };

        (controller, receiver)
    }

    pub fn send(&mut self, data: Vec<u8>) -> futures::StartSend<AppMessage, SendError<AppMessage>> {
        let msg_id = self.next_message_id;
        self.next_message_id += 1;
        println!("[AppController::send] {}", msg_id);
        self.sender.start_send(AppMessage::Send(msg_id as u32, std::u32::MAX, data))
    }

    pub fn reply(&mut self, id: u32, data: Vec<u8>) -> futures::StartSend<AppMessage, SendError<AppMessage>> {
        let msg_id = self.next_message_id;
        self.next_message_id += 1;
        self.sender.start_send(AppMessage::Send(msg_id as u32, id, data))
    }
}

fn handle_message(service_provider: Arc<Mutex<Box<ServiceProvider>>>, controller: &mut AppController, id: u32, method: &[u8], message: &[u8]) -> impl Future<Item = (), Error = std::io::Error> {
    match method {
        b"enum" => {
            let enum_sessions = EnumSessionsData {
                message: message.to_vec(),
            };
            print_network_message(Bytes::from(&enum_sessions.message[..]));
            service_provider.lock().unwrap()
                .enum_sessions(controller.clone(), id, enum_sessions)
        },
        b"open" => {
            let open = OpenData::parse(message);
            service_provider.lock().unwrap()
                .open(controller.clone(), id, open)
        },
        b"crpl" => {
            let create_player = CreatePlayerData::parse(message);
            service_provider.lock().unwrap()
                .create_player(controller.clone(), id, create_player)
        },
        b"repl" => {
            let reply = ReplyData::parse(message);
            print_network_message(Bytes::from(&reply.message[..]));
            service_provider.lock().unwrap()
                .reply(controller.clone(), id, reply)
        },
        b"send" => {
            let send = SendData::parse(message);
            print_network_message(Bytes::from(&send.message[..]));
            service_provider.lock().unwrap()
                .send(controller.clone(), id, send)
        },
        method => {
            println!("[HostServer::process_message] HostServer message: {} {:?}, {:?}", id, method, message);
            SPFuture::new(Box::new(future::finished(())))
        }
    }
}

fn handle_connection(service_provider: Arc<Mutex<Box<ServiceProvider>>>, sock: TcpStream) -> Result<()> {
    sock.set_nodelay(true)?;
    let (writer, reader) = Framed::new(sock, LengthDelimitedCodec::new()).split();
    let (mut app_controller, receiver) = AppController::create();
    println!("[handle_connection] Connection incoming");

    let read_future = reader.for_each(move |mut message| {
        let id = BigEndian::read_u32(&message.split_to(4));
        let _reply_id = BigEndian::read_u32(&message.split_to(4));
        let method = message.split_to(4);
        handle_message(Arc::clone(&service_provider), &mut app_controller, id, &method, &message)
    }).then(|result| {
        println!("[handle_connection] Connection finished");
        result
    }).map_err(|e| {
        eprintln!("[handle_connection] Request error: {:?}", e);
    });

    let app_messages = receiver.map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "this should never happen")
    }).map(|app_message| match app_message {
        AppMessage::Send(msg_id, reply_to_id, data) => {
            println!("[handle_connection] Send message {} in reply to {}", msg_id, reply_to_id);
            let mut message = BytesMut::with_capacity(data.len() + 12);
            message.put_u32_be(msg_id);
            message.put_u32_be(reply_to_id);
            message.put_u32_be(0);
            message.put(&data);
            message.freeze()
        },
    });

    let write_future = writer.send_all(app_messages).map_err(|e| {
        eprintln!("[handle_connection] Send app message error: {:?}", e);
    });

    let future = read_future.join(write_future)
        .map(|_| ());

    tokio::spawn(future);
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
    service_provider: Box<ServiceProvider>,
}

impl HostServer {
    pub fn new(port: u16, service_provider: Box<ServiceProvider>) -> Self {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let (controller, receiver) = ServerController::create();

        HostServer {
            address,
            controller,
            receiver,
            service_provider,
        }
    }

    pub fn start(self) -> Result<(impl Future<Item = (), Error = std::io::Error>, ServerController)> {
        println!("[HostServer::start] Starting HostServer on {:?}", self.address);
        let client = TcpListener::bind(&self.address)?;

        let service_provider = Arc::new(Mutex::new(self.service_provider));
        let server_controller = self.controller.clone();
        let control_messages = self.receiver
            .map(EventType::Control)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Control stream ended"));

        let server = client.incoming()
            .map(EventType::Socket)
            .select(control_messages)
            .map(|message| {
                println!("[HostServer::start] Receiving message: {:?}", message);
                message
            })
            .take_while(move |message| match message {
                EventType::Control(ControlMessage::Stop) => future::finished(false),
                _ => future::finished(true),
            })
            .filter_map(move |message| match message {
                EventType::Socket(sock) => Some(sock),
                _ => None,
            })
            .for_each(move |sock| {
                println!("[HostServer::start] Spawning socket handler...");
                future::result(handle_connection(Arc::clone(&service_provider), sock))
            });

        Ok((server, self.controller))
    }
}
