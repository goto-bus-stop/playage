use async_std::io;
use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use dprun::{structs::*, AppController, ServiceProvider, DPID, GUID};
use std::collections::HashMap;

#[allow(dead_code)]
const DPLAYI_PLAYER_SYSPLAYER: i32 = 1;
const DPLAYI_PLAYER_NAMESRVR: i32 = 2;
#[allow(dead_code)]
const DPLAYI_PLAYER_LOCAL: i32 = 8;

pub struct LocalOnlyServer {
    name_server: Option<AppController>,
    players: HashMap<GUID, AppController>,
    enumers: HashMap<DPID, AppController>,
}

impl LocalOnlyServer {
    pub fn make() -> Self {
        Self {
            name_server: None,
            players: HashMap::new(),
            enumers: HashMap::new(),
        }
    }

    pub fn set_name_server(&mut self, _id: GUID, controller: AppController) {
        self.name_server = Some(controller);
    }

    pub fn create_player(&mut self, id: GUID, controller: AppController) {
        self.players.insert(id, controller);
        log::trace!(
            "Current players: {:?}",
            self.players.keys().collect::<Vec<&GUID>>()
        );
    }

    pub async fn enum_sessions(&mut self, message: &[u8], requester: AppController) {
        self.enumers.insert(0, requester);
        match self.name_server {
            Some(ref mut name_server) => name_server.send(message.to_vec()).await,
            None => panic!("EnumSessions'd without a host"),
        };
    }

    async fn reply(&mut self, id: GUID, data: &[u8]) {
        match self.players.get_mut(&id) {
            Some(player) => {
                player.send(data.to_vec()).await;
            }
            None => {
                let futures = self
                    .enumers
                    .values_mut()
                    .map(|player| player.send(data.to_vec()));
                let _ = futures::future::join_all(futures).await;
            }
        }
    }

    async fn send(&mut self, to_player_id: Option<GUID>, data: &[u8]) {
        match to_player_id {
            Some(ref id) => {
                if let Some(player) = self.players.get_mut(id) {
                    player.send(data.to_vec()).await;
                }
            }
            None => match self.name_server {
                Some(ref mut name_server) => {
                    name_server.send(data.to_vec()).await;
                }
                None => panic!("Tried to send message to nonexistent name server"),
            },
        }
    }
}

pub struct LocalOnlySP {
    server: Arc<Mutex<LocalOnlyServer>>,
}

impl LocalOnlySP {
    pub fn new(server: Arc<Mutex<LocalOnlyServer>>) -> Self {
        Self { server }
    }
}

#[async_trait]
impl ServiceProvider for LocalOnlySP {
    async fn enum_sessions(
        &mut self,
        controller: AppController,
        _id: u32,
        data: EnumSessionsData,
    ) -> io::Result<()> {
        log::trace!(
            "[LocalOnlySP::enum_sessions] Got EnumSessions message: {:?}",
            data
        );
        self.server
            .lock()
            .await
            .enum_sessions(&data.message, controller)
            .await;
        Ok(())
    }

    async fn open(
        &mut self,
        _controller: AppController,
        _id: u32,
        data: OpenData,
    ) -> io::Result<()> {
        log::trace!("[LocalOnlySP::open] Got Open message: {:?}", data);
        Ok(())
    }

    async fn create_player(
        &mut self,
        controller: AppController,
        _id: u32,
        data: CreatePlayerData,
    ) -> io::Result<()> {
        log::trace!(
            "[LocalOnlySP::create_player] Got CreatePlayer message: {:?}",
            data
        );
        let mut server = self.server.lock().await;
        if data.flags & DPLAYI_PLAYER_NAMESRVR != 0 {
            server.set_name_server(data.player_guid, controller);
        } else {
            server.create_player(data.player_guid, controller);
        }
        Ok(())
    }

    async fn reply(
        &mut self,
        _controller: AppController,
        _id: u32,
        data: ReplyData,
    ) -> io::Result<()> {
        // log::trace!("[LocalOnlySP::reply] Got Reply message: {:?}", data);
        self.server
            .lock()
            .await
            .reply(data.reply_to, &data.message)
            .await;
        Ok(())
    }

    async fn send(
        &mut self,
        _controller: AppController,
        _id: u32,
        data: SendData,
    ) -> io::Result<()> {
        // log::trace!("[LocalOnlySP::send] Got Send message: {:?}", data);
        self.server
            .lock()
            .await
            .send(data.receiver_id, &data.message)
            .await;
        Ok(())
    }
}
