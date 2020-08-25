use async_std::prelude::*;
use dprun::{run, DPRunOptions, GUID};
// use dpsp_libp2p::Libp2pSP;
// use dpsp_local_only::{LocalOnlySP, LocalOnlyServer};
use std::time::Duration;

#[derive(PartialEq, Eq)]
enum SPType {
    TCPIP,
    #[allow(dead_code)]
    Local,
    #[allow(dead_code)]
    P2P,
}

/// Test app that sets up a DPChat session.
#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let dpchat = GUID::parse_str("5BFDB060-06A4-11D0-9C4F-00A0C905425E")?;
    let test_session_id = GUID::parse_str("5BFDB060-06A4-11D0-9C4F-00A0C905425E")?;

    let dprun_dir = std::env::current_dir()?.join("../dprun/bin/debug");

    let use_sp = SPType::TCPIP;

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
            unimplemented!()
            /*
            let local_server = Arc::new(Mutex::new(LocalOnlyServer::make()));

            host_options = host_options
                .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
                .named_address_part("INet", "127.0.0.1")
                .named_address_part("INetPort", 2197)
                .named_address_part("SelfID", host_guid.as_bytes());
            join_options = join_options
                .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
                .named_address_part("INet", "127.0.0.1")
                .named_address_part("INetPort", 2198)
                .named_address_part("SelfID", join_guid.as_bytes());
            */
        }
        SPType::P2P => {
            unimplemented!()
            /*
            host_options = host_options
                .service_provider_handler(Box::new(Libp2pSP::default()))
                .named_address_part("INet", "127.0.0.1")
                .named_address_part("INetPort", 2197)
                .named_address_part("SelfID", host_guid.as_bytes());
            join_options = join_options
                .service_provider_handler(Box::new(Libp2pSP::default()))
                .named_address_part("INet", "127.0.0.1")
                .named_address_part("INetPort", 2198)
                .named_address_part("SelfID", join_guid.as_bytes());
            */
        }
        SPType::TCPIP => {
            host_options = host_options
                .named_service_provider("TCPIP")
                .named_address_part("INet", "127.0.0.1");
            join_options = join_options
                .named_service_provider("TCPIP")
                .named_address_part("INet", "127.0.0.1");
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
    let join_instance = join.start().delay(Duration::from_secs(3));

    let (host_result, join_result) = host_instance.join(join_instance).await;
    let _ = host_result?;
    let _ = join_result?;

    println!("done");

    Ok(())
}
