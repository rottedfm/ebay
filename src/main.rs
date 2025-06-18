// -- src/main.rs
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use chrono::Local;
use clap::Parser;
use env_logger::Builder;
use log::{LevelFilter, info};

mod cli;
mod client;
mod csv;

use crate::cli::{Cli, Commands};
use crate::client::BrowserClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init logger
    setup_logger()?;

    // Parse cli args
    let cli = Cli::parse();

    // Load .env file into std::env
    dotenvy::dotenv().ok();

    // TODO: check if the we are in tty, If we are start virtual display

    // Load email creds from .env file (NOT ON GITHUB)
    let email = std::env::var("EBAY_EMAIL").context("Missing EBAY_EMAIL")?;
    let password = std::env::var("EBAY_PASSWORD").context("Missing EBAY_PASSWORD")?;

    // Match the subcommand provided via the Cli struct
    match &cli.command {
        // Inventory scraping workflow
        Commands::Inventory => {
            println!("📦 Starting inventory...");
            info!("Starting inventory...");

            // Init a new geckodriver client
            let mut browser = BrowserClient::new()
                .await
                .context("Failed to start browser client")?;

            // Begin login
            browser
                .goto("https://signin.ebay.com/")
                .await
                .context("Failed to navigate to eBay login page")?;

            browser
                .wait_if_captcha_detected()
                .await
                .context("Failed to wait for captcha")?;

            browser
                .email_submit(&email)
                .await
                .context("Failed to submit email")?;

            // small wait for scrolling
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            browser
                .password_submit(&password)
                .await
                .context("Failed to submit password")?;

            browser
                .wait_if_captcha_detected()
                .await
                .context("Failed to wait for captcha")?;

            // Navigate to eBay active listings
            browser
                .goto("https://www.ebay.com/mys/active")
                .await
                .context("Failed to navigate to eBay active listings page")?;

            // Trigger listing scraper
            browser
                .scrape_listings()
                .await
                .context("Failed to scrape listings")?;

            // Close geckodriver
            browser
                .quit()
                .await
                .context("Failed to close browser session")?;
        }

        // Send discount offer workflow
        Commands::Offer { percentage } => {
            info!("Offering customers {}% off...", percentage);

            let mut browser = BrowserClient::new()
                .await
                .context("Failed to start browser client")?;

            // Login
            browser
                .goto("https://signin.ebay.com/")
                .await
                .context("Failed to navigate to eBay login page")?;

            browser
                .wait_if_captcha_detected()
                .await
                .context("Failed to wait for captcha")?;

            browser
                .email_submit(&email)
                .await
                .context("Failed to submit email")?;

            browser
                .password_submit(&password)
                .await
                .context("Failed to submit password")?;

            // Navigate to overview page
            browser
                .goto("https://www.ebay.com/mys/overview")
                .await
                .context("Failed to reach overview")?;

            // Send discount offers to eligible listings
            browser
                .send_discount_offers(*percentage)
                .await
                .context("Failed to send discount offers")?;

            // TODO: Add check if offers were sent and add it to the csv

            // Close browser
            browser
                .quit()
                .await
                .context("Failed to close browser session")?;
        }
        &Commands::Profit => {
            let mut browser = BrowserClient::new()
                .await
                .context("Failed to start browser client")?;

            // Login
            browser
                .goto("https://signin.ebay.com/")
                .await
                .context("Failed to navigate to eBay login page")?;

            browser
                .wait_if_captcha_detected()
                .await
                .context("Failed to wait for captcha")?;

            browser
                .email_submit(&email)
                .await
                .context("Failed to submit email")?;

            browser
                .password_submit(&password)
                .await
                .context("Failed to submit password")?;

            browser
                .goto("https://www.ebay.com/mes/transactionlist")
                .await
                .context("Failed to navigate to payments page")?;

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            browser
                .find_profit()
                .await
                .context("Failed to find profit")?;

            browser
                .quit()
                .await
                .context("Failed to close browser session")?;
        }
    }
    Ok(())
}

// Inits the logger to write all log output to a timestamped file
fn setup_logger() -> anyhow::Result<()> {
    let log_dir = Path::new("logs");
    create_dir_all(log_dir)?;

    let now = Local::now().format("%d-%m-%dT%H-%M-%S").to_string();
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
