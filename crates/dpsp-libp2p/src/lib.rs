use async_std::io::{self, Read, Write};
use async_trait::async_trait;
use dprun::{structs, AppController, ServiceProvider};
use futures::future::BoxFuture;
use libp2p::{
    core::upgrade, core::UpgradeInfo, identity::Keypair, mdns::Mdns, InboundUpgrade, Multiaddr,
    OutboundUpgrade, Swarm,
};

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
    C: Read + Write,
{
    type Output = ();
    type Error = EnumSessionsError;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, _socket: C, _: Self::Info) -> Self::Future {
        Box::pin(async move { Ok(()) })
    }
}

impl<C> OutboundUpgrade<C> for EnumSessionsUpgrade
where
    C: Read + Write,
{
    type Output = ();
    type Error = EnumSessionsError;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, _socket: C, _: Self::Info) -> Self::Future {
        Box::pin(async move { Ok(()) })
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

#[async_trait]
impl ServiceProvider for Libp2pSP {
    async fn enum_sessions(
        &mut self,
        _controller: AppController,
        _id: u32,
        data: structs::EnumSessionsData,
    ) -> io::Result<()> {
        dbg!(&data);
        Ok(())
    }

    async fn open(
        &mut self,
        _controller: AppController,
        _id: u32,
        _data: structs::OpenData,
    ) -> io::Result<()> {
        let transport = libp2p::build_development_transport(self.local_key.clone())?;
        // how to make this work?
        // .upgrade(EnumSessionsUpgrade);
        let mut swarm = Swarm::new(
            transport,
            Mdns::new().unwrap(),
            self.local_key.public().into_peer_id(),
        );

        let _addr = Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        if let Some(dial_addr) = &self.address {
            Swarm::dial_addr(&mut swarm, dial_addr.clone()).unwrap();
        }

        async_std::task::spawn(async move {
            loop {
                let event = swarm.next_event().await;
                dbg!(event);
            }
        });

        Ok(())
    }

    async fn create_player(
        &mut self,
        _controller: AppController,
        _id: u32,
        _data: structs::CreatePlayerData,
    ) -> io::Result<()> {
        Ok(())
    }

    async fn reply(
        &mut self,
        _controller: AppController,
        _id: u32,
        _data: structs::ReplyData,
    ) -> io::Result<()> {
        Ok(())
    }

    async fn send(
        &mut self,
        _controller: AppController,
        _id: u32,
        _data: structs::SendData,
    ) -> io::Result<()> {
        Ok(())
    }
}
