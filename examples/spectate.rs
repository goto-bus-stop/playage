use aoc_spectate::SpectateStream;
use async_std::{
    fs::File,
    io::{Read, Write},
    net::TcpStream,
    task,
};
use std::{
    io,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    /// IP Address to connect to.
    address: String,
    /// Path to the Age of Empires 2 game directory.
    #[structopt(
        long = "game-path",
        short = "p",
        default_value = r"c:\Program Files (x86)\Microsoft Games\Age of Empires II"
    )]
    game_path: PathBuf,
}

#[cfg(target_os = "windows")]
fn start_aoc(basedir: &Path, game_name: &str, spec_file: &Path) -> io::Result<Child> {
    Command::new(basedir.join("Age2_x1/age2_x1.5.exe"))
        .arg(format!("GAME={}", game_name))
        .arg(format!(r#""{}""#, spec_file.to_string_lossy()))
        .spawn()
}

#[cfg(not(target_os = "windows"))]
fn start_aoc(basedir: &Path, game_name: &str, spec_file: &Path) -> io::Result<Child> {
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

    let aoc_path = basedir.join("Age2_x1/age2_x1.5.exe");
    Command::new("wine")
        .arg(aoc_path.to_string_lossy().to_string())
        .arg(format!("GAME={}", game_name))
        .arg(format!(r#""{}""#, to_wine(spec_file)))
        .spawn()
}

async fn amain(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:53754", args.address);
    let stream = TcpStream::connect(addr).await?;
    let mut sesh = SpectateStream::connect_stream(Box::new(stream)).await?;

    println!("Game: {}", sesh.game_name());
    println!("Ext: {}", sesh.file_type());
    println!("Streaming from: {}", sesh.player_name());

    let spec_file = args
        .game_path
        .join("SaveGame")
        .join(format!("spec.{}", sesh.file_type()));
    println!("{:?}", spec_file);
    let mut file = File::create(&spec_file).await?;
    let header = sesh.read_rec_header().await?;
    file.write_all(&header).await?;
    file.sync_data().await?;

    println!("Starting...");

    let running = Arc::new(AtomicBool::new(true));
    let thread = thread::spawn({
        let running = Arc::clone(&running);
        let game_name = sesh.game_name().to_string();
        move || {
            let mut aoc =
                start_aoc(&args.game_path, &game_name, &spec_file).expect("could not start aoc");
            let result = aoc.wait();
            running.store(false, Ordering::SeqCst);
            result.unwrap();
        }
    });

    println!("Receiving recorded game data...");

    let mut buffer = [0; 16 * 1024];
    while let Ok(num) = sesh.stream().read(&mut buffer).await {
        file.write_all(&buffer[0..num]).await?;
        file.sync_data().await?;
        if num == 0 {
            break;
        }
        if !running.load(Ordering::Relaxed) {
            println!("AoC exited! Stopping spec feed...");
            break;
        }
    }

    println!("No more actions! Waiting for AoC to close...");

    thread.join().unwrap();

    Ok(())
}

fn main() {
    let args = Cli::from_args();
    let task = task::spawn(async move {
        amain(args).await.unwrap();
    });
    task::block_on(task);
}
