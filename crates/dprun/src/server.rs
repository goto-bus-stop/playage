use std::mem;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::sync::mpsc::{channel, Sender, Receiver, SendError};
use tokio::io::Result;
use tokio::codec::{Framed, LengthDelimitedCodec};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use bytes::{BytesMut, BufMut, ByteOrder, BigEndian};
use crate::structs::*;

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

pub trait ServiceProvider {
    fn open(&mut self, _data: OpenData) {
    }
    fn create_player(&mut self, _data: CreatePlayerData) {
    }
}

#[derive(Clone)]
pub struct ServerController {
    sender: Sender<ControlMessage>,
}

impl ServerController {
    pub fn create() -> (Self, Receiver<ControlMessage>) {
        // TODO figure out appropriate buffer size
        // May only need to be oneshot
        let (sender, receiver) = channel(5);
        let controller = ServerController {
            sender,
        };

        (controller, receiver)
    }

    pub fn stop(&mut self) -> futures::StartSend<ControlMessage, SendError<ControlMessage>> {
        self.sender.start_send(ControlMessage::Stop)
    }
}

#[derive(Clone)]
pub struct AppController {
    sender: Sender<AppMessage>,
    next_message_id: Arc<AtomicUsize>,
}

impl AppController {
    pub fn create() -> (Self, Receiver<AppMessage>) {
        // TODO figure out appropriate buffer size
        let (sender, receiver) = channel(5);
        let controller = AppController {
            sender: sender,
            next_message_id: Arc::new(AtomicUsize::new(0)),
        };

        (controller, receiver)
    }

    pub fn send(&mut self, data: Vec<u8>) -> futures::StartSend<AppMessage, SendError<AppMessage>> {
        let msg_id = Arc::clone(&self.next_message_id).fetch_add(1, Ordering::Relaxed);
        self.sender.start_send(AppMessage::Send(msg_id as u32, std::u32::MAX, data))
    }

    pub fn reply(&mut self, id: u32, data: Vec<u8>) -> futures::StartSend<AppMessage, SendError<AppMessage>> {
        let msg_id = Arc::clone(&self.next_message_id).fetch_add(1, Ordering::Relaxed);
        self.sender.start_send(AppMessage::Send(msg_id as u32, id, data))
    }
}

macro_rules! cast_message {
    ($message:ident as $type:ty) => {
        {
            assert_eq!($message.len(), mem::size_of::<$type>());
            let mut buffer = [0; mem::size_of::<$type>()];
            buffer.copy_from_slice(&$message);
            let cast: $type = unsafe { mem::transmute(buffer) };
            cast
        }
    }
}

#[derive(Debug)]
enum EventType {
    Socket(TcpStream),
    Control(ControlMessage),
}

fn handle_message(controller: &mut AppController, id: u32, method: &[u8], message: &[u8]) {
    match method {
        b"enum" => {
            let enum_sessions = EnumSessionsData {
                message: message.to_vec(),
            };
            println!("[HostServer::process_message] Got EnumSessions message: {} {:?}", id, enum_sessions);
        },
        b"open" => {
            let open = cast_message!(message as OpenData);
            println!("[HostServer::process_message] Got Open message: {} {:?}", id, open);
        },
        b"crpl" => {
            let create_player = cast_message!(message as CreatePlayerData);
            println!("[HostServer::process_message] Got CreatePlayer message: {} {:?}", id, create_player);
            controller.reply(id, b"ok".to_vec());
        },
        method => {
            println!("[HostServer::process_message] HostServer message: {} {:?}, {:?}", id, method, message);
        }
    }
}

fn handle_connection(_controller: ServerController, sock: TcpStream) -> Result<()> {
    sock.set_nodelay(true)?;
    let (writer, reader) = Framed::new(sock, LengthDelimitedCodec::new()).split();
    let (mut app_controller, receiver) = AppController::create();
    println!("[handle_connection] Connection incoming");

    let read_future = reader.for_each(move |mut message| {
        let id = BigEndian::read_u32(&message.split_to(4));
        let method = message.split_to(4);
        handle_message(&mut app_controller, id, &method, &message);
        future::finished(())
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
            let mut message = BytesMut::with_capacity(data.len() + 8);
            message.put_u32_be(msg_id);
            message.put_u32_be(reply_to_id);
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

pub struct HostServer {
    address: SocketAddr,
    controller: ServerController,
    receiver: Receiver<ControlMessage>,
}

impl HostServer {
    pub fn new(port: u16) -> Self {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let (controller, receiver) = ServerController::create();

        HostServer {
            address,
            controller,
            receiver,
        }
    }

    pub fn start(self) -> Result<(impl Future<Item = (), Error = std::io::Error>, ServerController)> {
        println!("[HostServer::start] Starting HostServer on {:?}", self.address);
        let client = TcpListener::bind(&self.address)?;

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
                future::result(handle_connection(server_controller.clone(), sock))
            });

        Ok((server, self.controller))
    }
}
