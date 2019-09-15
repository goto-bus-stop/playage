use aoc_spectate::SpectateSession;
use async_std::{
    fs::File,
    io::{copy, Read, Write},
    net::TcpStream,
    task,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    thread,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(default_value = "c:\\Program Files (x86)\\Microsoft Games\\Age of Empires II")]
    game_path: PathBuf,
}

#[cfg(target_os = "windows")]
fn start_aoc(basedir: &Path, spec_file: &Path) {
}

#[cfg(not(target_os = "windows"))]
fn start_aoc(basedir: &Path, spec_file: &Path) {
    let mut child = Command::new("wine")
        .arg(basedir.join("Age2_x1/age2_x1.5.exe").to_string_lossy().to_string())
        .arg(format!(r#""{}""#, to_wine(spec_file)))
        .spawn()
        .expect("Could not start aoc");
    child.wait().unwrap();

    fn to_wine(path: &Path) -> String {
        let stdout = Command::new("winepath")
            .args(&["-w", &path.to_string_lossy()])
            .output()
            .expect("winepath failed")
            .stdout;
        std::str::from_utf8(&stdout)
            .expect("winepath spewed garbage")
            .trim()
            .to_string()
    }
}

async fn amain(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "192.168.178.16:53754";
    let stream = TcpStream::connect(addr).await?;
    let mut sesh = SpectateSession::connect_stream(Box::new(stream)).await?;

    println!("Game: {}", sesh.game_name());
    println!("Ext: {}", sesh.file_type());
    println!("Streaming from: {}", sesh.player_name());

    let spec_file = args.game_path
        .join("SaveGame")
        .join(format!("spec.{}", sesh.file_type()));
    println!("{:?}", spec_file);
    let mut file = File::create(&spec_file).await?;
    let (size, header) = sesh.read_rec_header().await?;
    file.write_all(&(size as u32).to_le_bytes()).await?;
    file.write_all(&header).await?;
    file.sync_data().await?;

    println!("Starting...");

    let thread = thread::spawn(move || {
        start_aoc(&args.game_path, &spec_file);
    });

    println!("Receiving recorded game data...");

    let mut buffer = [0; 16 * 1024];
    while let Ok(num) = sesh.stream().read(&mut buffer).await {
        file.write_all(&buffer[0..num]).await?;
        file.sync_data().await?;
        if num == 0 {
            break;
        }
    }

    println!("No more actions! Waiting for AoC to close...");

    thread.join();

    Ok(())
}

fn main() {
    let args = Cli::from_args();
    let task = task::spawn(async move {
        amain(args).await.unwrap();
    });
    task::block_on(task);
}
