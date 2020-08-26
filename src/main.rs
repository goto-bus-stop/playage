use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use dprun::{run, DPRunOptions, GUID};
use dpsp_libp2p::Libp2pSP;
use dpsp_local_only::{LocalOnlySP, LocalOnlyServer};
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, PartialEq, Eq)]
enum SPType {
    TCPIP,
    #[allow(dead_code)]
    Local,
    #[allow(dead_code)]
    P2P,
}

impl FromStr for SPType {
    type Err = &'static str;
    fn from_str(input: &str) -> Result<SPType, Self::Err> {
        match input {
            "tcp" => Ok(SPType::TCPIP),
            "local" => Ok(SPType::Local),
            "p2p" => Ok(SPType::P2P),
            _ => Err("unknown sp-type, must be tcp, local, p2p"),
        }
    }
}

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(long, short = "t", default_value = "tcp")]
    sp_type: SPType,
}

/// Test app that sets up a DPChat session.
#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let Cli { sp_type } = Cli::from_args();

    femme::with_level(femme::LevelFilter::Trace);

    let dpchat = GUID::parse_str("E9EB4143-0FA4-4E0B-BEB3-C5222657F9F2")?;
    let test_session_id = GUID::parse_str("5BFDB060-06A4-11D0-9C4F-00A0C905425E")?;

    let dprun_dir = std::env::current_dir()?.join("../dprun/bin/debug");

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

    match sp_type {
        SPType::Local => {
            let local_server = Arc::new(Mutex::new(LocalOnlyServer::make()));

            host_options = host_options
                .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
                .named_address_part("INet", "127.0.0.1")
                .named_address_part("INetPort", 2197)
                .named_address_part("SelfID", host_guid.as_bytes().to_vec());
            join_options = join_options
                .service_provider_handler(Box::new(LocalOnlySP::new(Arc::clone(&local_server))))
                .named_address_part("INet", "127.0.0.1")
                .named_address_part("INetPort", 2198)
                .named_address_part("SelfID", join_guid.as_bytes().to_vec());
        }
        SPType::P2P => {
            host_options = host_options
                .service_provider_handler(Box::new(Libp2pSP::default()))
                .named_address_part("INet", "127.0.0.1")
                .named_address_part("INetPort", 2197)
                .named_address_part("SelfID", host_guid.as_bytes().to_vec());
            join_options = join_options
                .service_provider_handler(Box::new(Libp2pSP::default()))
                .named_address_part("INet", "127.0.0.1")
                .named_address_part("INetPort", 2198)
                .named_address_part("SelfID", join_guid.as_bytes().to_vec());
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

    log::info!("Spawning dprun");
    log::info!("host CLI: {}", host.command());
    log::info!("join CLI: {}", join.command());

    let host_instance = host.start();
    let join_instance = join.start().delay(Duration::from_secs(3));

    let (host_result, join_result) = host_instance.join(join_instance).await;
    let _ = host_result?;
    let _ = join_result?;

    log::info!("done");

    Ok(())
}
