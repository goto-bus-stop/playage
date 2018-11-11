use tokio::prelude::*;
use dprun::{run, DPAddressValue, DPRunOptions, GUID};

/// Test app that sets up a DPChat session.
fn main() {
    let dpchat = GUID(0x5BFDB060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);
    let test_session_id = GUID(0x5BFDB060, 0x06A4, 0x11D0, 0x9C, 0x4F, 0x00, 0xA0, 0xC9, 0x05, 0x42, 0x5E);

    let host_options = DPRunOptions::new()
        .host(Some(test_session_id))
        .player_name("Hosting".into())
        .application(dpchat)
        .named_service_provider("TCPIP")
        .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
        // .named_address_part("INetPort", DPAddressValue::Number(2197))
        .cwd("/home/goto-bus-stop/Code/aocmulti/dprun/bin/debug".into())
        .finish();

    let join_options = DPRunOptions::new()
        .join(test_session_id)
        .player_name("Joining".into())
        .application(dpchat)
        .named_service_provider("TCPIP")
        .named_address_part("INet", DPAddressValue::String("127.0.0.1".to_string()))
        // .named_address_part("INetPort", DPAddressValue::Number(2197))
        .cwd("/home/goto-bus-stop/Code/aocmulti/dprun/bin/debug".into())
        .finish();

    let host = run(host_options);
    let join = run(join_options);

    println!("Spawning dprun");
    let host_instance = host.start();
    let join_instance = join.start();

    let future = host_instance.join(join_instance)
        .map(|_| ())
        .map_err(|e| eprintln!("error: {:?}", e));

    tokio::run(future);

    println!("done");
}
