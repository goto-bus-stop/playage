use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::prelude::*;
use tokio::timer::Delay;
use dprun::{
    run,
    structs::*,
    DPID, GUID,
    ServiceProvider, AppController,
    DPAddressValue, DPRunOptions,
};

struct LocalOnlyServer {
    host: Option<AppController>,
    players: HashMap<DPID, AppController>,
}

impl LocalOnlyServer {
    pub fn make() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            host: None,
            players: HashMap::new(),
        }))
    }

    pub fn create_player(&mut self, id: DPID, controller: AppController) {
        if self.host.is_none() {
            self.host = Some(controller);
        } else {
            self.players.insert(id, controller);
        }
    }

    pub fn enum_sessions(&mut self, message: &[u8]) {
        match self.host {
            Some(ref mut host) => host.send(message.to_vec()),
            None => panic!("EnumSessions'd without a host"),
        };
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
    fn enum_sessions(&mut self, _controller: AppController, _id: u32, data: EnumSessionsData) {
        println!("[LocalOnlySP::enum_sessions] Got EnumSessions message: {:?}", data);
        self.server.lock().unwrap()
            .enum_sessions(&data.message)
    }

    fn open(&mut self, _controller: AppController, _id: u32, data: OpenData) {
        println!("[LocalOnlySP::open] Got Open message: {:?}", data);
    }

    fn create_player(&mut self, controller: AppController, _id: u32, data: CreatePlayerData) {
        println!("[LocalOnlySP::create_player] Got CreatePlayer message: {:?}", data);
        self.server.lock().unwrap()
            .create_player(data.player_id, controller)
    }
}

/// Test app that sets up a DPChat session.
fn main() {
    let dpchat = GUID(0x5BFD_B060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);
    let test_session_id = GUID(0x5BFD_B060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);

    let local_server = LocalOnlyServer::make();

    let host_options = DPRunOptions::builder()
        .host(Some(test_session_id))
        .player_name("Hosting".into())
        .application(dpchat)
        .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
        .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
        .named_address_part("INetPort", DPAddressValue::Number(2197))
        .cwd("/home/goto-bus-stop/Code/aocmulti/dprun/bin/debug".into())
        .finish();

    let join_options = DPRunOptions::builder()
        .join(test_session_id)
        .player_name("Joining".into())
        .application(dpchat)
        .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
        .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
        .named_address_part("INetPort", DPAddressValue::Number(2198))
        .cwd("/home/goto-bus-stop/Code/aocmulti/dprun/bin/debug".into())
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
