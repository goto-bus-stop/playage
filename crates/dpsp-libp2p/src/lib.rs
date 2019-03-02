use futures::{prelude::*, future::poll_fn};
use tokio::prelude::*;
use libp2p::{
    Multiaddr,
    Swarm,
    mdns::Mdns,
    secio::SecioKeyPair,
};
use dprun::{ServiceProvider, AppController, SPFuture, structs};

pub struct Libp2pSP {
    local_key: SecioKeyPair,
    address: Option<Multiaddr>,
    // swarm: Option<Swarm>,
}

impl Libp2pSP {
    pub fn new() -> Self {
        Self {
            local_key: SecioKeyPair::ed25519_generated().unwrap(),
            address: None,
        }
    }

    pub fn with_address(self, address: Multiaddr) -> Self {
        Self { address: Some(address), ..self }
    }

    // TODO allow constructing a dpsp-libp2p instance from an existing (multiplex) connection
    // pub fn from(transport: CommonTransport) {
    // }
}

impl ServiceProvider for Libp2pSP {
    fn enum_sessions(&mut self, controller: AppController, _id: u32, data: structs::EnumSessionsData) -> SPFuture {
        println!("[Libp2pSP::enum_sessions] {:?}", data);
        SPFuture::new(Box::new(future::finished(())))
    }

    fn open(&mut self, controller: AppController, _id: u32, data: structs::OpenData) -> SPFuture {
        println!("[Libp2pSP::open]");
        let transport = libp2p::build_tcp_ws_secio_mplex_yamux(self.local_key.clone());
        let mut swarm = Swarm::new(
            transport,
            Mdns::new().unwrap(),
            self.local_key.to_peer_id());

        let addr = Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
        println!("[Libp2pSP::open] {:?}", addr);

        if let Some(dial_addr) = &self.address {
            Swarm::dial_addr(&mut swarm, dial_addr.clone()).unwrap();
        }

        SPFuture::new(Box::new(poll_fn(move || {
            swarm.poll().expect("Error polling swarm");
            Ok(Async::NotReady)
        })))
    }

    fn create_player(&mut self, controller: AppController, _id: u32, data: structs::CreatePlayerData) -> SPFuture {
        SPFuture::new(Box::new(future::finished(())))
    }

    fn reply(&mut self, controller: AppController, _id: u32, data: structs::ReplyData) -> SPFuture {
        SPFuture::new(Box::new(future::finished(())))
    }

    fn send(&mut self, controller: AppController, _id: u32, data: structs::SendData) -> SPFuture {
        SPFuture::new(Box::new(future::finished(())))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
