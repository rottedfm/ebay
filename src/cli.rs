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
    /// Login to eBay and save cookies till they expire
    Login,
    /// Scrape and save montly profit in a loop for led matrix
    Profit,
    /// Send offers at set percentage
    Offer { percentage: f32 },
    /// Scrape inventory data
    Inventory,
    /// Scrape and save to DB
    Sync,
}
