use dprun::{structs, AppController, SPFuture, ServiceProvider};
use futures::{future::poll_fn, future::FutureResult, prelude::*};
use libp2p::{
    core::upgrade, core::UpgradeInfo, identity::Keypair, mdns::Mdns, InboundUpgrade, Multiaddr,
    OutboundUpgrade, Swarm,
};
use tokio::prelude::*;

#[derive(Debug)]
pub enum EnumSessionsError {
    Any,
}

impl std::fmt::Display for EnumSessionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EnumSessionsError::Any => write!(f, "EnumSessionsError::Any"),
        }
    }
}

unsafe impl Send for EnumSessionsError {}
unsafe impl Sync for EnumSessionsError {}

impl std::error::Error for EnumSessionsError {}

#[derive(Debug, Default, Clone, Copy)]
struct EnumSessionsUpgrade;
impl UpgradeInfo for EnumSessionsUpgrade {
    type Info = &'static [u8];
    type InfoIter = std::iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        std::iter::once(b"/dpsp-enum/0.0.0")
    }
}

impl<C> InboundUpgrade<C> for EnumSessionsUpgrade
where
    C: AsyncRead + AsyncWrite,
{
    type Output = ();
    type Error = EnumSessionsError;
    type Future = FutureResult<Self::Output, Self::Error>;

    fn upgrade_inbound(self, i: upgrade::Negotiated<C>, _: Self::Info) -> Self::Future {
        future::ok(())
    }
}

impl<C> OutboundUpgrade<C> for EnumSessionsUpgrade
where
    C: AsyncRead + AsyncWrite,
{
    type Output = ();
    type Error = EnumSessionsError;
    type Future = FutureResult<Self::Output, Self::Error>;

    fn upgrade_outbound(self, i: upgrade::Negotiated<C>, _: Self::Info) -> Self::Future {
        future::ok(())
    }
}

pub struct Libp2pSP {
    local_key: Keypair,
    address: Option<Multiaddr>,
    // swarm: Option<Swarm>,
}

impl Default for Libp2pSP {
    fn default() -> Self {
        Self {
            local_key: Keypair::generate_ed25519(),
            address: None,
        }
    }
}

impl Libp2pSP {
    pub fn with_address(self, address: Multiaddr) -> Self {
        Self {
            address: Some(address),
            ..self
        }
    }

    // TODO allow constructing a dpsp-libp2p instance from an existing (multiplex) connection
    // pub fn from(transport: CommonTransport) {
    // }
}

impl ServiceProvider for Libp2pSP {
    fn enum_sessions(
        &mut self,
        _controller: AppController,
        _id: u32,
        data: structs::EnumSessionsData,
    ) -> SPFuture {
        dbg!(&data);
        SPFuture::new(Box::new(future::finished(())))
    }

    fn open(&mut self, _controller: AppController, _id: u32, _data: structs::OpenData) -> SPFuture {
        let transport = libp2p::build_development_transport(self.local_key.clone());
        // how to make this work?
        // .with_upgrade(EnumSessionsUpgrade);
        let mut swarm = Swarm::new(
            transport,
            Mdns::new().unwrap(),
            self.local_key.public().into_peer_id(),
        );

        let _addr = Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        if let Some(dial_addr) = &self.address {
            Swarm::dial_addr(&mut swarm, dial_addr.clone()).unwrap();
        }

        SPFuture::new(Box::new(poll_fn(move || {
            swarm.poll().expect("Error polling swarm");
            Ok(Async::NotReady)
        })))
    }

    fn create_player(
        &mut self,
        _controller: AppController,
        _id: u32,
        _data: structs::CreatePlayerData,
    ) -> SPFuture {
        SPFuture::new(Box::new(future::finished(())))
    }

    fn reply(
        &mut self,
        _controller: AppController,
        _id: u32,
        _data: structs::ReplyData,
    ) -> SPFuture {
        SPFuture::new(Box::new(future::finished(())))
    }

    fn send(&mut self, _controller: AppController, _id: u32, _data: structs::SendData) -> SPFuture {
        SPFuture::new(Box::new(future::finished(())))
    }
}
