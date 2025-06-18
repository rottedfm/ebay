use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ebay")]
#[command(about = "eBay bot manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Send offers at set percentage
    Offer { percentage: i16 },
    /// Scrape inventory data
    Inventory,
    /// Fetch stats
    Stats,
}
