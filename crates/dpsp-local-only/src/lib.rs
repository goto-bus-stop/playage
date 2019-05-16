use dprun::{structs::*, AppController, SPFuture, ServiceProvider, DPID, GUID};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::prelude::*;

const DPLAYI_PLAYER_SYSPLAYER: i32 = 1;
const DPLAYI_PLAYER_NAMESRVR: i32 = 2;
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
        println!(
            "Current players: {:?}",
            self.players.keys().collect::<Vec<&GUID>>()
        );
    }

    pub fn enum_sessions(&mut self, message: &[u8], requester: AppController) {
        self.enumers.insert(0, requester);
        match self.name_server {
            Some(ref mut name_server) => name_server.send(message.to_vec()),
            None => panic!("EnumSessions'd without a host"),
        };
    }

    fn reply(&mut self, id: GUID, data: &[u8]) {
        match self.players.get_mut(&id) {
            Some(player) => {
                player.send(data.to_vec());
            }
            None => {
                self.enumers.values_mut().for_each(|player| {
                    player.send(data.to_vec());
                });
            }
        }
    }

    fn send(&mut self, to_player_id: Option<GUID>, data: &[u8]) {
        match to_player_id {
            Some(ref id) => {
                self.players.get_mut(id).map(|player| {
                    player.send(data.to_vec());
                });
            }
            None => match self.name_server {
                Some(ref mut name_server) => {
                    name_server.send(data.to_vec());
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

fn immediately() -> SPFuture {
    SPFuture::new(Box::new(future::finished(())))
}

impl ServiceProvider for LocalOnlySP {
    fn enum_sessions(
        &mut self,
        controller: AppController,
        _id: u32,
        data: EnumSessionsData,
    ) -> SPFuture {
        // println!("[LocalOnlySP::enum_sessions] Got EnumSessions message: {:?}", data);
        self.server
            .lock()
            .unwrap()
            .enum_sessions(&data.message, controller);
        immediately()
    }

    fn open(&mut self, _controller: AppController, _id: u32, data: OpenData) -> SPFuture {
        println!("[LocalOnlySP::open] Got Open message: {:?}", data);
        immediately()
    }

    fn create_player(
        &mut self,
        controller: AppController,
        _id: u32,
        data: CreatePlayerData,
    ) -> SPFuture {
        println!(
            "[LocalOnlySP::create_player] Got CreatePlayer message: {:?}",
            data
        );
        if data.flags & DPLAYI_PLAYER_NAMESRVR != 0 {
            self.server
                .lock()
                .unwrap()
                .set_name_server(data.player_guid, controller)
        } else {
            self.server
                .lock()
                .unwrap()
                .create_player(data.player_guid, controller)
        }
        immediately()
    }

    fn reply(&mut self, controller: AppController, _id: u32, data: ReplyData) -> SPFuture {
        // println!("[LocalOnlySP::reply] Got Reply message: {:?}", data);
        self.server
            .lock()
            .unwrap()
            .reply(data.reply_to, &data.message);
        immediately()
    }

    fn send(&mut self, controller: AppController, id: u32, data: SendData) -> SPFuture {
        // println!("[LocalOnlySP::send] Got Send message: {:?}", data);
        self.server
            .lock()
            .unwrap()
            .send(data.receiver_id, &data.message);
        immediately()
    }
}
