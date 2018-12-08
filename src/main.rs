mod test_sp;

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use tokio::prelude::*;
use tokio::timer::Delay;
use rand::{thread_rng, Rng};
use dprun::{
    run,
    GUID,
    DPAddressValue,
    DPRunOptions,
};
use crate::test_sp::{
    LocalOnlyServer,
    LocalOnlySP,
};

/// Test app that sets up a DPChat session.
fn main() {
    let dpchat = GUID(0x5BFD_B060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);
    let test_session_id = GUID(0x5BFD_B060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);

    let dprun_dir = std::env::current_dir()
        .unwrap()
        .join("../dprun/bin/debug");

    let use_custom_sp = true;

    let mut host_options = DPRunOptions::builder()
        .host(Some(test_session_id))
        .player_name("Hosting".into())
        .application(dpchat)
        .cwd(dprun_dir.clone());

    let mut join_options = DPRunOptions::builder()
        .join(test_session_id)
        .player_name("Joining".into())
        .application(dpchat)
        .cwd(dprun_dir.clone());

    if use_custom_sp {
        let local_server = Arc::new(Mutex::new(
                LocalOnlyServer::make()));

        let mut host_guid = [0u8; 16];
        let mut join_guid = [0u8; 16];
        thread_rng().fill(&mut host_guid);
        thread_rng().fill(&mut join_guid);

        host_options = host_options
            .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
            .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
            .named_address_part("INetPort", DPAddressValue::Number(2197))
            .named_address_part("SelfID", DPAddressValue::Binary(host_guid.to_vec()));
        join_options = join_options
            .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
            .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
            .named_address_part("INetPort", DPAddressValue::Number(2198))
            .named_address_part("SelfID", DPAddressValue::Binary(join_guid.to_vec()));
    } else {
        host_options = host_options
            .named_service_provider("TCPIP")
            .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()));
        join_options = join_options
            .named_service_provider("TCPIP")
            .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()));
    }

    let host_options = host_options.finish();
    let join_options = join_options.finish();

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
