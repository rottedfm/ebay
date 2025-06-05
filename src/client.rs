use crate::csv::write_listings_to_csv;
use anyhow::{Context, Result};
use fantoccini::{Client as FantocciniClient, ClientBuilder, Locator, elements::Element};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::{Child, Command, Stdio};
use std::{fs, time::Duration};
use tempfile::TempDir;
use tokio::time::sleep;

pub struct BrowserClient {
    driver: Child,
    client: FantocciniClient,
    _profile_dir: TempDir, // Keeps Firefox profile alive
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Listing {
    pub title: String,
    pub item_id: String,
    pub price: String,
    pub views: String,
    pub watchers: String,
}

impl BrowserClient {
    pub async fn new() -> Result<Self> {
        let geckodriver_path =
            which::which("geckodriver").context("Could not find 'geckodriver' in PATH")?;

        let profile_dir =
            Self::create_firefox_profile().context("Failed to create custom Firefox profile")?;

        info!("Starting geckodriver with custom Firefox profile...");
        let driver = Command::new(&geckodriver_path)
            .arg("--port")
            .arg("4444")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn geckodriver process")?;

        sleep(Duration::from_secs(2)).await;

        info!("Connecting to Fantoccini WebDriver...");
        let client = ClientBuilder::native()
            .connect("http://localhost:4444")
            .await
            .context("Failed to connect to geckodriver on port 4444")?;

        info!("BrowserClient initialized.");
        Ok(Self {
            driver,
            client,
            _profile_dir: profile_dir,
        })
    }

    fn create_firefox_profile() -> Result<TempDir> {
        let dir = tempfile::tempdir().context("Failed to create temporary profile dir")?;
        let user_js_path = dir.path().join("user.js");

        debug!("Creating custom Firefox profile at {:?}", dir.path());

        fs::write(
            &user_js_path,
            r#"
user_pref("general.useragent.override", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/122.0.0.0 Safari/537.36");
user_pref("privacy.resistFingerprinting", false);
user_pref("dom.webdriver.enabled", false);
user_pref("useAutomationExtension", false);
"#,
        )
        .context("Failed to write user.js to Firefox profile")?;

        Ok(dir)
    }

    pub async fn goto(&mut self, url: &str) -> Result<()> {
        info!("Navigating to: {url}");
        self.client.goto(url).await.context("Failed to navigate")?;
        Ok(())
    }

    pub async fn wait_if_captcha_detected(&mut self) -> Result<()> {
        let current_url = self.client.current_url().await?.to_string();

        if current_url.to_lowercase().contains("captcha") {
            println!("⚠️ CAPTCHA detected. Please solve it manually in the browser...");

            // Poll until URL changes away from captcha
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let new_url = self.client.current_url().await?.to_string();

                if new_url != current_url && !new_url.to_lowercase().contains("captcha") {
                    println!("✅ CAPTCHA cleared. Continuing...");
                    break;
                } else {
                    println!("🔄 Still on CAPTCHA page: {}", new_url);
                }
            }
        }

        Ok(())
    }

    pub async fn email_submit(&mut self, email: &str) -> Result<()> {
        info!("Typing email into selector: #userid...");

        let username: Element = self
            .client
            .wait()
            .for_element(Locator::Css("#userid"))
            .await
            .context("Failed to wait for #userid")?;

        username
            .send_keys(email)
            .await
            .context("Failed to send_keys to #userid")?;

        self.client
            .find(Locator::Css("#signin-continue-btn"))
            .await
            .context("Failed to find #signin-continue-btn")?
            .click()
            .await
            .context("Failed to click #sigin-continue-btn")?;

        Ok(())
    }

    pub async fn find_profit(&mut self) -> Result<String> {
        let funds: Element = self
            .client
            .wait()
            .for_element(Locator::Css(".payment-tile--positive > div:nth-child(1) > div:nth-child(1) > span:nth-child(2) > a:nth-child(1) > span:nth-child(1) > span:nth-child(1) > span:nth-child(1) > span:nth-child(1)"))
            .await
            .context("Failed to wait for .payment-tile--positive > div:nth-child(1) > div:nth-child(1) > span:nth-child(2) > a:nth-child(1) > span:nth-child(1) > span:nth-child(1) > span:nth-child(1) > span:nth-child(1)")?;

        let total_funds = funds
            .text()
            .await
            .context("Failed to get total funds value")?;

        Ok(total_funds)
    }

    pub async fn scroll_to_element(&mut self, selector: &str) -> Result<()> {
        // Find the element using the provided CSS selector
        let elem = self
            .client
            .find(Locator::Css(selector))
            .await
            .with_context(|| format!("Failed to find element with selector: {}", selector))?;

        // JavaScript code to scroll the element into view
        let js_script = r#"
            arguments[0].scrollIntoView({ behavior: 'smooth', block: 'center' });
        "#;

        // Execute the JavaScript with the element as an argument
        self.client
            .execute(js_script, vec![json!(elem)])
            .await
            .context("Failed to execute scrollIntoView JavaScript")?;

        Ok(())
    }

    pub async fn password_submit(&mut self, password: &str) -> Result<()> {
        info!("Typing password into selector: #userid...");

        let pass: Element = self
            .client
            .wait()
            .for_element(Locator::Css("#pass"))
            .await
            .context("Failed to wait for #pass")?;

        self.scroll_to_element("#pass")
            .await
            .context("Failed to scroll to element")?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        pass.click().await.context("Failed to click #pass")?;

        pass.send_keys(password)
            .await
            .context("Failed to send keys to #pass")?;

        self.client
            .find(Locator::Css("#sgnBt"))
            .await
            .context("Failed to find #sgnBt")?
            .click()
            .await
            .context("Failed to click #sgnBt")?;

        Ok(())
    }

    pub async fn scrape_listings(&mut self) -> Result<Vec<Listing>> {
        let items = self
            .client
            .find_all(Locator::Css("div.active-item"))
            .await
            .context("Failed to find listing elements")?;

        let mut listings = Vec::new();

        for (_i, item) in items.into_iter().enumerate() {
            let title = item
                .find(Locator::Css("h3.item-title span"))
                .await
                .context("Failed to find title element")?
                .text()
                .await
                .unwrap_or_else(|_| "<missing title>".into());

            let item_id = item
                .find(Locator::Css(".item__itemid span.normal"))
                .await
                .context("Failed to find item ID")?
                .text()
                .await
                .unwrap_or_else(|_| "<missing item ID>".into())
                .replace("Item ID: ", "");

            let price = item
                .find(Locator::Css(".item__price span.bold"))
                .await
                .context("Failed to find price")?
                .text()
                .await
                .unwrap_or_else(|_| "<missing price>".into());

            let views = item
                .find(Locator::Css(
                    ".me-item-activity__column:nth-child(1) .me-item-activity__column-count",
                ))
                .await
                .context("Failed to find views count")?
                .text()
                .await
                .unwrap_or_else(|_| "0".into());

            let watchers = item
                .find(Locator::Css(
                    ".me-item-activity__column:nth-child(2) .me-item-activity__column-count",
                ))
                .await
                .context("Failed to find watchers count")?
                .text()
                .await
                .unwrap_or_else(|_| "0".into());

            listings.push(Listing {
                title,
                item_id,
                price,
                views,
                watchers,
            });

            for (i, listing) in listings.iter().enumerate() {
                info!("Listing {} raw: {:?}", i + 1, listing);

                println!("----------------------------------------");
                println!("📦 Listing #{}", i + 1);
                println!("📝 Title   : {}", listing.title);
                println!("🆔 Item ID : {}", listing.item_id);
                println!("💲 Price   : {}", listing.price);
                println!("👀 Views   : {}", listing.views);
                println!("⭐ Watchers: {}", listing.watchers);
            }
        }

        write_listings_to_csv(&listings, "output/listings.csv")?;

        Ok(listings)
    }

    pub async fn quit(mut self) -> Result<()> {
        info!("Shutting down browser and geckodriver...");
        if let Err(e) = self.client.close().await {
            error!("Failed to close WebDriver session: {e}");
        }
        if let Err(e) = self.driver.kill() {
            error!("Failed to kill geckodriver: {e}");
        }
        Ok(())
    }
}
