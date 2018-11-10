use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicBool};
use tokio::io::Result;
use tokio::codec::{Framed, LengthDelimitedCodec};
use tokio::net::TcpListener;
use tokio::prelude::*;

// TODO replace with a sender/receiver oneshot channel
pub struct StopToken {
    keep_going: Arc<AtomicBool>,
}

impl StopToken {
    pub fn new() -> StopToken {
        StopToken {
            keep_going: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn clone(&self) -> Self {
        StopToken {
            keep_going: Arc::clone(&self.keep_going),
        }
    }

    pub fn keep_going(&self) -> bool {
        self.keep_going.load(Ordering::Relaxed)
    }

    pub fn stop(&self) -> () {
        self.keep_going.store(false, Ordering::Relaxed)
    }
}

struct MessageParser {
}
impl MessageParser {
    pub fn new() -> Self {
        MessageParser {}
    }

    pub fn process(&self, method: &[u8], message: &[u8]) {
        match method {
            b"init" => {
                println!("Got init message");
            },
            method => {
                println!("HostServer message: {:?}, {:?}", method, message);
            }
        }
    }
}

pub struct HostServer {
    address: SocketAddr,
    parser: MessageParser,
}

impl HostServer {
    pub fn new(port: u16) -> HostServer {
        HostServer {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port),
            parser: MessageParser::new(),
        }
    }

    pub fn start(self) -> Result<(impl Future<Item = (), Error = std::io::Error>, StopToken)> {
        let client = TcpListener::bind(&self.address)?;

        let listening = StopToken::new();
        let stop_listening = listening.clone();
        let server = client.incoming()
            .take_while(move |_| {
                // this doesn't work because it's only called once a new client connects
                println!("checking take_while...");
                future::finished(listening.keep_going())
            })
            .for_each(move |sock| {
                let (writer, reader) = Framed::new(sock, LengthDelimitedCodec::new()).split();
                println!("HostServer incoming");
                reader.for_each(move |message| {
                    let method = &message[0..4];
                    MessageParser::new()
                        .process(method, &message[4..]);
                    future::finished(())
                }).then(|result| {
                    println!("HostServer finished");
                    result
                })
            });

        Ok((server, stop_listening))
    }
}
