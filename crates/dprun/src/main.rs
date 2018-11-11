use tokio::prelude::*;
use dprun::{run, DPAddressPart, DPRunOptions, GUID};

fn main() -> Result<(), std::io::Error> {
    let dpchat = GUID(0x5BFDB060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);
    // let tcpip = GUID(0x36E95EE0, 0x8577, 0x11cf, 0x96, 0x0c, 0x00, 0x80, 0xc7, 0x53, 0x4e, 0x82);
    let dprunsp = GUID(0xb1ed2367, 0x609b, 0x4c5c, 0x87, 0x55, 0xd2, 0xa2, 0x9b, 0xb9, 0xa5, 0x54);
    let inet = GUID(0xc4a54da0, 0xe0af, 0x11cf, 0x9c, 0x4e, 0x00, 0xa0, 0xc9, 0x05, 0x42, 0x5e);
    let inet_port = GUID(0xe4524541, 0x8ea5, 0x11d1, 0x8a, 0x96, 0x00, 0x60, 0x97, 0xb0, 0x14, 0x11);

    let options = DPRunOptions::new()
        .host(None)
        .player_name("Test".into())
        .application(dpchat)
        .service_provider(dprunsp)
        .address_part(DPAddressPart::String(inet, "127.0.0.1".into()))
        .address_part(DPAddressPart::Number(inet_port, 2197))
        .cwd("/home/goto-bus-stop/Code/aocmulti/dprun/bin/debug".into())
        .finish();

    let dp_run = run(options);

    println!("Spawning dprun");
    let instance = dp_run.start();
    tokio::run(instance.map_err(|e| eprintln!("error: {:?}", e)));
    println!("done");
    Ok(())

}
