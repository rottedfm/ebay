mod cli;
mod scraper;

use clap::Parser;
use log::info;

use crate::cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .init();

    match &cli.command {
        Some(Commands::Inventory { scrape }) => {
            if *scrape {
                info!("Starting inventory scraper... ")
            }
        }
        None => {}
    }
}
