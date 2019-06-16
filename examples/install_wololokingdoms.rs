use wololokingdoms::{ConvertOptions, ConvertListener, Converter};
use std::path::PathBuf;
use structopt::StructOpt;

/// WololoKingdoms Installer CLI.
#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(long = "hd-path")]
    hd_path: PathBuf,
    #[structopt(long = "install-path")]
    install_path: PathBuf,
}

fn main() {
    let args = Cli::from_args();

    let settings = ConvertOptions::builder()
        .hd_path(&args.hd_path)
        .voobly_path(&args.install_path)
        .up_path(&args.install_path)
        .resource_path(&PathBuf::from("./third_party/wololokingdoms/resources"))
        .dlc_level(3)
        .build();

    let mut listener = ConvertListener::default();
    listener.on_log(|text: &str| {
        println!("log from rust callback: {}", text);
    });

    let mut converter = Converter::new(settings, listener);
    converter.run().unwrap();
}
