use progress::Bar;
use std::path::PathBuf;
use structopt::StructOpt;
use wololokingdoms::{ConvertListener, ConvertOptions, Converter};

/// WololoKingdoms Installer CLI.
#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(long = "hd-path")]
    hd_path: PathBuf,
    #[structopt(long = "install-path")]
    install_path: PathBuf,
}

struct ProgressListener {
    bar: Bar,
}

impl ConvertListener for ProgressListener {
    fn log(&mut self, _text: &str) {
        // println!("log from rust callback: {}", text);
    }

    fn set_info(&mut self, text: &str) {
        self.bar.set_job_title(text);
    }

    fn progress(&mut self, progress: f32) {
        self.bar.reach_percent((progress * 100.0) as i32);
    }

    fn finished(&mut self) {
        self.bar.jobs_done();
    }
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

    let listener = Box::new(ProgressListener { bar: Bar::new() });
    let mut converter = Converter::new(settings, listener);
    converter.run().unwrap();
}
