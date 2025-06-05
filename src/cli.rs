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
    /// Scrape and save montly profit in a loop for led matrix
    Counter,
    /// Send offers at set percentage
    Offer { percentage: f32 },
    /// Scrape inventory data
    Inventory,
}
