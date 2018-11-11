use std::mem;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::sync::mpsc::{channel, Sender, Receiver, SendError};
use tokio::io::Result;
use tokio::codec::{Framed, LengthDelimitedCodec};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use bytes::{ByteOrder, BigEndian};
use crate::structs::*;

#[derive(Debug)]
pub enum ControlMessage {
    /// Stop the server.
    Stop,
    /// Send a message to the DirectPlay application.
    Send(u32, u32, Vec<u8>),
}

pub trait ServiceProvider {
    fn open(&mut self, data: OpenData) {
    }
    fn create_player(&mut self, data: CreatePlayerData) {
    }
}

#[derive(Clone)]
pub struct Controller {
    sender: Sender<ControlMessage>,
    next_message_id: Arc<AtomicUsize>,
}

impl Controller {
    pub fn create() -> (Self, Receiver<ControlMessage>) {
        // TODO figure out appropriate buffer size
        let (sender, receiver) = channel(5);
        let controller = Controller {
            sender: sender,
            next_message_id: Arc::new(AtomicUsize::new(0)),
        };

        (controller, receiver)
    }

    pub fn send(&mut self) -> () {
    }

    pub fn reply(&mut self, id: u32, data: Vec<u8>) -> futures::StartSend<ControlMessage, SendError<ControlMessage>> {
        let msg_id = Arc::clone(&self.next_message_id).fetch_add(1, Ordering::Relaxed);
        self.sender.start_send(ControlMessage::Send(msg_id as u32, std::u32::MAX, data))
    }

    pub fn stop(&mut self) -> futures::StartSend<ControlMessage, SendError<ControlMessage>> {
        self.sender.start_send(ControlMessage::Stop)
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

struct MessageParser {
}

impl MessageParser {
    pub fn new() -> Self {
        MessageParser {}
    }

    pub fn process(&self, controller: &mut Controller, id: u32, method: &[u8], message: &[u8]) {
        match method {
            b"open" => {
                let open = cast_message!(message as OpenData);
                println!("Got Open message: {} {:?}", id, open);
            },
            b"crpl" => {
                let create_player = cast_message!(message as CreatePlayerData);
                println!("Got CreatePlayer message: {} {:?}", id, create_player);
                controller.reply(id, b"ok".to_vec());
            },
            method => {
                println!("HostServer message: {} {:?}, {:?}", id, method, message);
            }
        }
    }
}

#[derive(Debug)]
enum EventType {
    Socket(TcpStream),
    Control(ControlMessage),
}

pub struct HostServer {
    address: SocketAddr,
    controller: Controller,
    receiver: Receiver<ControlMessage>,
}

impl HostServer {
    pub fn new(port: u16) -> Self {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let (controller, receiver) = Controller::create();

        HostServer {
            address,
            controller,
            receiver,
        }
    }

    fn process(mut controller: Controller, sock: TcpStream) -> () {
        let parser = Arc::new(MessageParser::new());
        let (writer, reader) = Framed::new(sock, LengthDelimitedCodec::new()).split();
        println!("Connection incoming incoming");
        let future = reader.for_each(move |mut message| {
            let id = BigEndian::read_u32(&message.split_to(4));
            let method = message.split_to(4);
            Arc::clone(&parser)
                .process(&mut controller, id, &method, &message);
            future::finished(())
        }).then(|result| {
            println!("Connection finished");
            result
        });

        tokio::spawn(future.map_err(|e| eprintln!("Request error: {:?}", e)));
    }

    pub fn start(self) -> Result<(impl Future<Item = (), Error = std::io::Error>, Controller)> {
        let client = TcpListener::bind(&self.address)?;

        let controller = self.controller;
        let return_controller = controller.clone();
        let control_messages = self.receiver
            .map(EventType::Control)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Control stream ended"));

        let server = client.incoming()
            .map(EventType::Socket)
            .select(control_messages)
            .take_while(move |message| match message {
                EventType::Control(ControlMessage::Stop) => future::finished(false),
                _ => future::finished(true),
            })
            .filter_map(move |message| match message {
                EventType::Socket(sock) => Some(sock),
                _ => None,
            })
            .for_each(move |sock| {
                Self::process(controller.clone(), sock);
                future::finished(())
            });

        Ok((server, return_controller))
    }
}
