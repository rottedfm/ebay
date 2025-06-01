use chrono::Local;
use clap::Parser;
use env_logger::Builder;
use log::{LevelFilter, info};
use std::fs::{OpenOptions, create_dir_all};
use std::io::{self, Write};
use std::path::Path;

mod cli;

use crate::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::Login => {
            info!("Starting login...");
            // TODO: Implement login
        }
        Commands::Inventory => {
            info!("Starting inventory scraper...");
            // TODO: Implement inventory scraper
        }
        Commands::Profit => {
            info!("Starting profit updater thread...");
            // TODO: Implement profit updater thread
        }
        Commands::Sync => {
            info!("Starting sync (inventory + db save)...");
            // TODO: implement full flow
        }
        Commands::Offer { percentage } => {
            info!("Offering customers {}% off...", percentage);
            // TODO: Offer a set percentage off to customers
        }
    }
    Ok(())
}

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
    // Ensures log directory exists
    let log_dir = Path::new("logs");
    create_dir_all(log_dir)?;

    // Create timestamped log file
    let now = Local::now().format("%Y-%m-%dT%H-%M-%S").to_string();
    let log_path = log_dir.join(format!("{}.log", now));
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let stdout = io::stdout();
    let multi_writer = MultiWriter::new(stdout, file);

    Builder::new()
        .filter(None, LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .format(move |buf, record| {
            writeln!(
                buf,
                "[{}] {} - {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(multi_writer)))
        .init();

    println!("📝 Logging to: {:?}", log_path);
    Ok(())
}

// Utility to write logs to stdout AND file
struct MultiWriter<W1, W2> {
    w1: W1,
    w2: W2,
}

impl<W1, W2> MultiWriter<W1, W2> {
    fn new(w1: W1, w2: W2) -> Self {
        Self { w1, w2 }
    }
}

impl<W1: Write, W2: Write> Write for MultiWriter<W1, W2> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.w1.write_all(buf)?;
        self.w2.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w1.flush()?;
        self.w2.flush()?;
        Ok(())
    }
}
