use dprun::{run, DPAddressValue, DPRunOptions, GUID};
use dpsp_libp2p::Libp2pSP;
use dpsp_local_only::{LocalOnlySP, LocalOnlyServer};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{prelude::*, timer::Delay};

#[derive(PartialEq, Eq)]
enum SPType {
    TCPIP,
    Local,
    P2P,
}

/// Test app that sets up a DPChat session.
fn main() {
    let dpchat = GUID::parse_str("5BFDB060-06A4-11D0-9C4F-00A0C905425E").unwrap();
    let test_session_id = GUID::parse_str("5BFDB060-06A4-11D0-9C4F-00A0C905425E").unwrap();

    let dprun_dir = std::env::current_dir().unwrap().join("../dprun/bin/debug");

    let use_sp = SPType::P2P;

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

    let host_guid = GUID::new_v4();
    let join_guid = GUID::new_v4();

    match use_sp {
        SPType::Local => {
            let local_server = Arc::new(Mutex::new(LocalOnlyServer::make()));

            host_options = host_options
                .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
                .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
                .named_address_part("INetPort", DPAddressValue::Number(2197))
                .named_address_part(
                    "SelfID",
                    DPAddressValue::Binary(host_guid.as_bytes().to_vec()),
                );
            join_options = join_options
                .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
                .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
                .named_address_part("INetPort", DPAddressValue::Number(2198))
                .named_address_part(
                    "SelfID",
                    DPAddressValue::Binary(join_guid.as_bytes().to_vec()),
                );
        }
        SPType::P2P => {
            host_options = host_options
                .service_provider_handler(Box::new(Libp2pSP::default()))
                .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
                .named_address_part("INetPort", DPAddressValue::Number(2197))
                .named_address_part(
                    "SelfID",
                    DPAddressValue::Binary(host_guid.as_bytes().to_vec()),
                );
            join_options = join_options
                .service_provider_handler(Box::new(Libp2pSP::default()))
                .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
                .named_address_part("INetPort", DPAddressValue::Number(2198))
                .named_address_part(
                    "SelfID",
                    DPAddressValue::Binary(join_guid.as_bytes().to_vec()),
                );
        }
        SPType::TCPIP => {
            host_options = host_options
                .named_service_provider("TCPIP")
                .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()));
            join_options = join_options
                .named_service_provider("TCPIP")
                .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()));
        }
    }

    let host_options = host_options.finish();
    let join_options = join_options.finish();

    let host = run(host_options);
    let join = run(join_options);

    println!("Spawning dprun");
    println!("host CLI: {}", host.command());
    println!("join CLI: {}", join.command());

    let host_instance = host.start();
    let join_instance = Delay::new(Instant::now() + Duration::from_secs(3)).then(|_| join.start());

    let future = host_instance
        .join(join_instance)
        .map(|_| ())
        .map_err(|e| eprintln!("error: {:?}", e));

    tokio::run(future);

    println!("done");
}
