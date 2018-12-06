use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::prelude::*;
use tokio::timer::Delay;
use rand::{thread_rng, Rng};
use dprun::{
    run,
    structs::*,
    DPID, GUID,
    ServiceProvider, AppController,
    DPAddressValue, DPRunOptions,
};

const DPLAYI_PLAYER_SYSPLAYER: i32 = 1;
const DPLAYI_PLAYER_NAMESRVR: i32 = 2;
const DPLAYI_PLAYER_LOCAL: i32 = 8;

struct LocalOnlyServer {
    name_server: Option<AppController>,
    players: HashMap<GUID, AppController>,
    enumers: HashMap<DPID, AppController>,
}

impl LocalOnlyServer {
    pub fn make() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            name_server: None,
            players: HashMap::new(),
            enumers: HashMap::new(),
        }))
    }

    pub fn set_name_server(&mut self, _id: GUID, controller: AppController) {
        self.name_server = Some(controller);
    }

    pub fn create_player(&mut self, id: GUID, controller: AppController) {
        self.players.insert(id, controller);
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
            },
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
            },
            None => match self.name_server {
                Some(ref mut name_server) => {
                    name_server.send(data.to_vec());
                },
                None => panic!("Tried to send message to nonexistent name server"),
            },
        }
    }
}

struct LocalOnlySP {
    server: Arc<Mutex<LocalOnlyServer>>,
}

impl LocalOnlySP {
    pub fn new(server: Arc<Mutex<LocalOnlyServer>>) -> Self {
        Self { server }
    }
}

impl ServiceProvider for LocalOnlySP {
    fn enum_sessions(&mut self, controller: AppController, _id: u32, data: EnumSessionsData) {
        println!("[LocalOnlySP::enum_sessions] Got EnumSessions message: {:?}", data);
        self.server.lock().unwrap()
            .enum_sessions(&data.message, controller)
    }

    fn open(&mut self, _controller: AppController, _id: u32, data: OpenData) {
        println!("[LocalOnlySP::open] Got Open message: {:?}", data);
    }

    fn create_player(&mut self, controller: AppController, _id: u32, data: CreatePlayerData) {
        println!("[LocalOnlySP::create_player] Got CreatePlayer message: {:?}", data);
        if data.flags & DPLAYI_PLAYER_NAMESRVR != 0 {
            self.server.lock().unwrap()
                .set_name_server(data.player_guid, controller)
        } else {
            self.server.lock().unwrap()
                .create_player(data.player_guid, controller)
        }
    }

    fn reply(&mut self, controller: AppController, _id: u32, data: ReplyData) {
        println!("[LocalOnlySP::reply] Got Reply message: {:?}", data);
        self.server.lock().unwrap()
            .reply(data.reply_to, &data.message)
    }

    fn send(&mut self, controller: AppController, id: u32, data: SendData) {
        println!("[LocalOnlySP::send] Got Send message: {:?}", data);
        self.server.lock().unwrap()
            .send(data.receiver_id, &data.message)
    }
}

/// Test app that sets up a DPChat session.
fn main() {
    let dpchat = GUID(0x5BFD_B060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);
    let test_session_id = GUID(0x5BFD_B060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);

    let local_server = LocalOnlyServer::make();

    let dprun_dir = std::env::current_dir()
        .unwrap()
        .join("../dprun/bin/debug");

    let mut host_guid = [0u8; 16];
    let mut join_guid = [0u8; 16];
    thread_rng().fill(&mut host_guid);
    thread_rng().fill(&mut join_guid);
    let host_guid = host_guid.to_vec();
    let join_guid = join_guid.to_vec();

    let host_options = DPRunOptions::builder()
        .host(Some(test_session_id))
        .player_name("Hosting".into())
        .application(dpchat)
        .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
        .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
        .named_address_part("INetPort", DPAddressValue::Number(2197))
        .named_address_part("SelfID", DPAddressValue::Binary(host_guid))
        .cwd(dprun_dir.clone())
        .finish();

    let join_options = DPRunOptions::builder()
        .join(test_session_id)
        .player_name("Joining".into())
        .application(dpchat)
        .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
        .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
        .named_address_part("INetPort", DPAddressValue::Number(2198))
        .named_address_part("SelfID", DPAddressValue::Binary(join_guid))
        .cwd(dprun_dir.clone())
        .finish();

    let host = run(host_options);
    let join = run(join_options);

    println!("Spawning dprun");
    println!("host CLI: {}", host.command());
    println!("join CLI: {}", join.command());

    let host_instance = host.start();
    let join_instance = Delay::new(Instant::now() + Duration::from_secs(3))
        .then(|_| join.start());

    let future = host_instance.join(join_instance)
        .map(|_| ())
        .map_err(|e| eprintln!("error: {:?}", e));

    tokio::run(future);

    println!("done");
}
