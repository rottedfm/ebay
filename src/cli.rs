use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = 
r#"          
          $$\                           
          $$ |                          
 $$$$$$\  $$$$$$$\   $$$$$$\  $$\   $$\ 
$$  __$$\ $$  __$$\  \____$$\ $$ |  $$ |
$$$$$$$$ |$$ |  $$ | $$$$$$$ |$$ |  $$ |
$$   ____|$$ |  $$ |$$  __$$ |$$ |  $$ |
\$$$$$$$\ $$$$$$$  |\$$$$$$$ |\$$$$$$$ |
 \_______|\_______/  \_______| \____$$ |
                              $$\   $$ |
                              \$$$$$$  |
                               \______/ "#, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
  /// Runs invertory query or scraper.
  Inventory {
    /// Scrapes ebay listings and stores the data in an sqlite database
    #[arg(short, long)]
    scrape: bool, 
  }

}
