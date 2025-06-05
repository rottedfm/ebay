use anyhow::Context;
use chrono::Local;
use clap::Parser;
use env_logger::Builder;
use log::{LevelFilter, info};
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::Path;

mod cli;
mod client;
mod csv;

use crate::cli::{Cli, Commands};
use crate::client::BrowserClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger()?;

    let cli = Cli::parse();

    match &cli.command {
        Commands::Inventory => {
            println!("📦 Starting inventory...");
            info!("Starting inventory...");

            let mut browser = BrowserClient::new()
                .await
                .context("Failed to start browser client")?;

            browser
                .goto("https://signin.ebay.com/")
                .await
                .context("Failed to navigate to eBay login page")?;

            browser
                .wait_if_captcha_detected()
                .await
                .context("Failed to wait for captcha")?;

            browser
                .email_submit("rottedfm@proton.me")
                .await
                .context("Failed to submit email")?;

            browser
                .password_submit("mqh8y~?+g{fpQl7S")
                .await
                .context("Failed to submit password")?;

            browser
                .goto("https://www.ebay.com/mys/active")
                .await
                .context("Failed to navigate to eBay active listings page")?;

            browser
                .scrape_listings()
                .await
                .context("Failed to scrape listings")?;

            browser
                .quit()
                .await
                .context("Failed to close browser session")?;
        }
        Commands::Counter => {
            println!("📈 Starting profit updater...");
            info!("Starting profit updater...");

            let mut browser = BrowserClient::new()
                .await
                .context("Failed to start browser client")?;

            browser
                .goto("https://signin.ebay.com/")
                .await
                .context("Failed to navigate to eBay login page")?;

            browser
                .wait_if_captcha_detected()
                .await
                .context("Failed to wait for captcha")?;

            browser
                .email_submit("rottedfm@proton.me")
                .await
                .context("Failed to submit email")?;

            browser
                .password_submit("mqh8y~?+g{fpQl7S")
                .await
                .context("Failed to submit password")?;

            browser
                .goto("https://ppcapp.ebay.com/myppc/wallet/list")
                .await
                .context("Failed to navigate to eBay wallet page")?;

            let total_funds = browser
                .find_profit()
                .await
                .context("Failed to find monthly profit")?;

            println!("💰 Total funds: {}", total_funds);

            browser
                .quit()
                .await
                .context("Failed to close browser session")?;
        }
        Commands::Offer { percentage } => {
            info!("Offering customers {}% off...", percentage);

            let mut browser = BrowserClient::new()
                .await
                .context("Failed to start browser client")?;

            browser
                .goto("https://signin.ebay.com/")
                .await
                .context("Failed to navigate to eBay login page")?;

            browser
                .wait_if_captcha_detected()
                .await
                .context("Failed to wait for captcha")?;

            browser
                .email_submit("rottedfm@proton.me")
                .await
                .context("Failed to submit email")?;

            browser
                .password_submit("mqh8y~?+g{fpQl7S")
                .await
                .context("Failed to submit password")?;

            browser
                .goto("https://www.ebay.com/mys/overview")
                .await
                .context("Failed to reach overview")?;

            browser
                .send_discount_offers(*percentage)
                .await
                .context("Failed to send discount offers")?;

            browser
                .quit()
                .await
                .context("Failed to close browser session")?;
        }
    }
    Ok(())
}
fn setup_logger() -> anyhow::Result<()> {
    let log_dir = Path::new("logs");
    create_dir_all(log_dir)?;

    let now = Local::now().format("%Y-%m-%dT%H-%M-%S").to_string();
    let log_path = log_dir.join(format!("{}.log", now));
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    Builder::new()
        .filter(None, LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {} - {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();

    println!("📝 Logging only to file: {:?}", log_path);
    Ok(())
}
