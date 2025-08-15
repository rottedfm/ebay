use crate::event::{AppEvent, Event, EventHandler};
use fantoccini::{Client, ClientBuilder};
use log::{error, info};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    DefaultTerminal,
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command};
use chrono::Utc;

#[derive(Debug, Default, Clone)]
pub struct ScrollState {
    pub vertical_scroll: usize,
}

impl ScrollState {
    pub fn scroll_down(&mut self) {
        self.vertical_scroll += 1;
    }
    
    pub fn scroll_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
    }
    
    pub fn scroll_page_down(&mut self) {
        self.vertical_scroll += 10;
    }
    
    pub fn scroll_page_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(10);
    }
    
    pub fn scroll_to_top(&mut self) {
        self.vertical_scroll = 0;
    }
    
    pub fn scroll_to_bottom(&mut self) {
        self.vertical_scroll = 1000; // Large value to scroll to bottom
    }
}

/// Represents an eBay listing with all relevant information for CSV export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    /// The title of the listing
    pub title: String,
    /// The current price of the item
    pub price: String,
    /// Shipping cost information
    pub shipping: Option<String>,
    /// Item condition (New, Used, etc.)
    pub condition: Option<String>,
    /// Number of watchers for this item
    pub watchers: Option<u32>,
    /// Seller username
    pub seller: Option<String>,
    /// Seller feedback score
    pub seller_feedback: Option<String>,
    /// Whether the item has a "Buy It Now" option
    pub buy_it_now: bool,
    /// Whether the item accepts "Best Offer"
    pub accepts_offers: bool,
    /// Item location
    pub location: Option<String>,
    /// Number of items available
    pub quantity_available: Option<u32>,
    /// Whether this is a new listing
    pub is_new_listing: bool,
    /// Item URL or ID for reference
    pub item_id: Option<String>,
    /// Item URL for direct access
    pub url: Option<String>,
    /// Any additional notes or features
    pub notes: Vec<String>,
    /// Item specifics (brand, model, color, etc.) as key-value pairs
    pub item_specifics: Vec<String>,
    /// Item description from seller
    pub description: Option<String>,
}

impl Default for Listing {
    fn default() -> Self {
        Self {
            title: String::new(),
            price: String::new(),
            shipping: None,
            condition: None,
            watchers: None,
            seller: None,
            seller_feedback: None,
            buy_it_now: false,
            accepts_offers: false,
            location: None,
            quantity_available: None,
            is_new_listing: false,
            item_id: None,
            url: None,
            notes: Vec::new(),
            item_specifics: Vec::new(),
            description: None,
        }
    }
}

/// Application state representing different phases of the eBay scraping process.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
    /// The application is loading.
    #[default]
    Loading,
    /// The application is running.
    Running,
}

/// Represents the current view mode of the scrollview widget.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum ScrollViewMode {
    /// Display paragraph view with seller stats and info.
    #[default]
    Paragraph,
    /// Display table view with listings.
    Table,
}

/// Main application structure managing the eBay scraper state and WebDriver interactions.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// The application state.
    pub state: AppState,
    /// Event handler.
    pub events: EventHandler,
    /// The WebDriver client for browser automation.
    pub client: Option<Client>,
    /// The geckodriver process.
    pub geckodriver: Option<Child>,
    /// Progress value from 0.0 to 1.0 representing scraping completion.
    pub progress: f64,
    /// Current status message displayed to user.
    pub progress_message: String,
    /// eBay seller's feedback score (e.g., "99.1% positive").
    pub feedback_score: Option<String>,
    /// Number of items sold by the eBay seller.
    pub items_sold: Option<u32>,
    /// Number of followers for the eBay seller.
    pub follower_count: Option<u32>,
    /// Whether a CAPTCHA challenge is currently active.
    pub captcha_detected: bool,
    /// Whether the app is waiting for user interaction (e.g., solving CAPTCHA).
    pub waiting_for_user_input: bool,
    /// Scraped eBay listings.
    pub listings: Vec<Listing>,
    /// Selected listing index for table navigation
    pub selected_listing_index: usize,
    /// Scroll offset for table display
    pub scroll_offset: usize,
    /// Current scrollview mode (paragraph or table)
    pub scroll_view_mode: ScrollViewMode,
    /// Scroll offset for paragraph view
    pub paragraph_scroll_offset: usize,
    /// ScrollState for the main scrollview widget
    pub scroll_view_state: ScrollState,
    /// Whether the user has locked to a specific section (true = locked)
    pub section_locked: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            state: AppState::default(),
            events: EventHandler::new(),
            client: None,
            geckodriver: None,
            progress: 0.0,
            progress_message: String::new(),
            feedback_score: None,
            items_sold: None,
            follower_count: None,
            captcha_detected: false,
            waiting_for_user_input: false,
            listings: Vec::new(),
            selected_listing_index: 0,
            scroll_offset: 0,
            scroll_view_mode: ScrollViewMode::default(),
            paragraph_scroll_offset: 0,
            scroll_view_state: ScrollState::default(),
            section_locked: false,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.events.send(AppEvent::Connect);
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit().await?,
                    AppEvent::Connect => self.connect().await?,
                    AppEvent::ClientReady => {
                        self.events.send(AppEvent::Init(
                            "https://www.ebay.com/usr/thriftngo5".to_string(),
                        ));
                    }
                    AppEvent::ScrapeFeedback(feedback_text) => {
                        self.feedback_score = Some(feedback_text.clone());
                        info!(
                            "Received feedback text: {}",
                            self.feedback_score.as_deref().unwrap_or("N/A")
                        );
                    }
                    AppEvent::ScrapeItemsSold(items_sold_count) => {
                        self.items_sold = Some(items_sold_count);
                        info!(
                            "Received items sold count: {}",
                            self.items_sold.unwrap_or(0)
                        );
                    }
                    AppEvent::SetProgress(progress, message) => {
                        self.progress = progress;
                        self.progress_message = message;
                    }
                    AppEvent::Init(url) => {
                        self.navigate_to_public_page(url.clone()).await?;
                        self.start_captcha_monitoring().await?;
                    }
                    AppEvent::ScrapeFollowerCount(follower_count) => {
                        self.follower_count = Some(follower_count);
                        info!(
                            "Received follower count: {}",
                            self.follower_count.unwrap_or(0)
                        );
                    }
                    AppEvent::GeckodriverStarted => {
                        info!("Geckodriver started successfully");
                        self.events.send(AppEvent::ClientReady);
                    }
                    AppEvent::GeckodriverError(error) => {
                        self.progress_message = format!("Geckodriver error: {}", error);
                    }
                    AppEvent::WebDriverConnected => {
                        info!("WebDriver client connected");
                        self.events.send(AppEvent::ClientReady);
                    }
                    AppEvent::WebDriverError(error) => {
                        self.progress_message = format!("WebDriver error: {}", error);
                    }
                    AppEvent::NavigateToUrl(url) => {
                        info!("Navigating to URL: {}", url);
                    }
                    AppEvent::NavigationComplete => {
                        info!("Navigation completed successfully");
                    }
                    AppEvent::NavigationError(error) => {
                        self.progress_message = format!("Navigation error: {}", error);
                    }
                    AppEvent::CaptchaDetected => {
                        info!("🚨 CAPTCHA detected - waiting for user to solve");
                        self.captcha_detected = true;
                        self.waiting_for_user_input = true;
                        self.events.send(AppEvent::SetProgress(
                            self.progress,
                            "⚠️  CAPTCHA detected! Please solve it manually, then it will automatically continue...".to_string(),
                        ));
                    }
                    AppEvent::CaptchaResolved => {
                        info!("✅ CAPTCHA resolved - continuing scraping");
                        self.captcha_detected = false;
                        self.waiting_for_user_input = false;
                        
                        let client = self.client.clone();
                        let sender = self.events.sender.clone();
                        
                        tokio::spawn(async move {
                            let _ = sender.send(Event::App(AppEvent::SetProgress(
                                0.4,
                                "📦 Scraping items sold...".to_string(),
                            )));
                            
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            
                            if let Some(client) = &client {
                                match Self::scrape_items_sold_static(&client).await {
                                    Ok(items_sold) => {
                                        let _ = sender.send(Event::App(AppEvent::ScrapeItemsSold(items_sold)));
                                    }
                                    Err(e) => {
                                        log::error!("Failed to scrape items sold: {}", e);
                                    }
                                }
                            }
                            
                            let _ = sender.send(Event::App(AppEvent::SetProgress(
                                0.6,
                                "⭐ Scraping feedback score...".to_string(),
                            )));
                            
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            
                            if let Some(client) = &client {
                                match Self::scrape_feedback_static(&client).await {
                                    Ok(feedback_score) => {
                                        let _ = sender.send(Event::App(AppEvent::ScrapeFeedback(feedback_score)));
                                    }
                                    Err(e) => {
                                        log::error!("Failed to scrape feedback: {}", e);
                                    }
                                }
                            }
                            
                            let _ = sender.send(Event::App(AppEvent::SetProgress(
                                0.8,
                                "👥 Scraping follower count...".to_string(),
                            )));
                            
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            
                            if let Some(client) = &client {
                                match Self::scrape_follower_count_static(&client).await {
                                    Ok(follower_count) => {
                                        let _ = sender.send(Event::App(AppEvent::ScrapeFollowerCount(follower_count)));
                                    }
                                    Err(e) => {
                                        log::error!("Failed to scrape follower count: {}", e);
                                    }
                                }
                            }
                            
                            let _ = sender.send(Event::App(AppEvent::SetProgress(
                                0.9,
                                "🖱️ Clicking \'See All\' button...".to_string(),
                            )));

                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                            if let Some(client) = &client {
                                match Self::click_see_all_button_static(&client).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        log::error!("Failed to click \'See All\' button: {}", e);
                                    }
                                }
                            }

                            let _ = sender.send(Event::App(AppEvent::SetProgress(
                                0.95,
                                "📋 Scraping listings...".to_string(),
                            )));
                            
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            
                            if let Some(client) = &client {
                                match Self::scrape_active_listings(&client).await {
                                    Ok(listings) => {
                                        let _ = sender.send(Event::App(AppEvent::ScrapeListings(listings)));
                                    }
                                    Err(e) => {
                                        log::error!("Failed to scrape listings: {}", e);
                                    }
                                }
                            }
                            
                            let _ = sender.send(Event::App(AppEvent::SetProgress(
                                1.0,
                                "✅ Scraping complete!".to_string(),
                            )));
                            
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            
                            let _ = sender.send(Event::App(AppEvent::ScrapingComplete));
                        });
                    }
                    AppEvent::ScrapingComplete => {
                        self.state = AppState::Running;
                    }
                    AppEvent::ScrapeListings(listings) => {
                        self.listings = listings.clone();
                        // Reset selection to first item when new listings are loaded
                        self.selected_listing_index = 0;
                        self.scroll_offset = 0;
                        info!("Received {} scraped listings", listings.len());
                        
                        // Trigger enrichment of listings
                        self.events.send(AppEvent::EnrichListings);
                    }
                    AppEvent::EnrichListings => {
                        let client = self.client.clone();
                        let sender = self.events.sender.clone();
                        let mut listings = self.listings.clone();
                        
                        tokio::spawn(async move {
                            let _ = sender.send(Event::App(AppEvent::SetProgress(
                                0.95,
                                "🔍 Enriching listings with detailed information...".to_string(),
                            )));
                            
                            if let Some(client) = &client {
                                // Enrich each listing with detailed information
                                let total_listings = listings.len();
                                for (index, listing) in listings.iter_mut().enumerate() {
                                    let _ = sender.send(Event::App(AppEvent::SetProgress(
                                        0.95 + (0.04 * (index as f64 / total_listings as f64)),
                                        format!("🔍 Processing listing {}/{}: {}", 
                                               index + 1, total_listings, 
                                               &listing.title.chars().take(30).collect::<String>()),
                                    )));
                                    
                                    // Construct URL from item_id
                                    if let Some(item_id) = &listing.item_id {
                                        let item_url = format!("https://www.ebay.com/itm/{}", item_id);
                                        
                                        if let Ok((item_specifics, description)) = Self::scrape_item_details(&client, &item_url).await {
                                            listing.item_specifics = item_specifics;
                                            listing.description = description;
                                        }
                                        
                                        // Small delay between requests
                                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                                    }
                                }
                            }
                            
                            // Send the enriched listings for saving
                            let _ = sender.send(Event::App(AppEvent::EnrichedListings(listings)));
                        });
                    }
                    AppEvent::EnrichedListings(listings) => {
                        self.listings = listings.clone();
                        // Ensure selection is still valid
                        if self.selected_listing_index >= self.listings.len() && !self.listings.is_empty() {
                            self.selected_listing_index = self.listings.len() - 1;
                            // Adjust scroll offset accordingly
                            self.scroll_offset = if self.selected_listing_index >= 19 {
                                self.selected_listing_index - 19
                            } else {
                                0
                            };
                        }
                        info!("Received {} enriched listings", listings.len());
                        
                        let filename = format!("ebay_listings_{}.csv", 
                            Utc::now().format("%Y%m%d_%H%M%S"));
                        
                        if let Err(e) = self.save_listings_to_csv(&filename) {
                            error!("Failed to save listings to CSV: {}", e);
                        } else {
                            info!("Successfully saved {} listings to {}", listings.len(), filename);
                        }
                        
                        self.events.send(AppEvent::SetProgress(1.0, "✅ Scraping complete!".to_string()));
                        let _ = tokio::spawn(async {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        });
                        self.events.send(AppEvent::ScrapingComplete);
                    }
                    AppEvent::ClickSeeAll => {
                        let client = self.client.clone();
                        tokio::spawn(async move {
                            if let Some(client) = &client {
                                match Self::click_see_all_button_static(&client).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        log::error!("Failed to click see all button: {}", e);
                                    }
                                }
                            }
                        });
                    }
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Enter => {
                if self.section_locked {
                    // If locked, unlock and allow normal scrolling
                    self.section_locked = false;
                } else {
                    // Lock to current section
                    self.section_locked = true;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.section_locked {
                    // If locked to a section, handle navigation within that section
                    match self.scroll_view_mode {
                        ScrollViewMode::Paragraph => {
                            // Scroll down in paragraph view
                            self.paragraph_scroll_offset += 1;
                        }
                        ScrollViewMode::Table => {
                            // Navigate table rows
                            if !self.listings.is_empty() && self.selected_listing_index < self.listings.len() - 1 {
                                self.selected_listing_index += 1;
                                // Keep selection visible - scroll down if needed
                                let visible_rows = 25; // Max visible rows
                                if self.selected_listing_index >= self.scroll_offset + visible_rows {
                                    self.scroll_offset = self.selected_listing_index - visible_rows + 1;
                                }
                            }
                        }
                    }
                } else {
                    // If not locked, scroll the entire scrollview
                    self.scroll_view_state.scroll_down();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.section_locked {
                    // If locked to a section, handle navigation within that section
                    match self.scroll_view_mode {
                        ScrollViewMode::Paragraph => {
                            // Scroll up in paragraph view
                            self.paragraph_scroll_offset = self.paragraph_scroll_offset.saturating_sub(1);
                        }
                        ScrollViewMode::Table => {
                            // Navigate table rows
                            if self.selected_listing_index > 0 {
                                self.selected_listing_index -= 1;
                                // Keep selection visible - scroll up if needed
                                if self.selected_listing_index < self.scroll_offset {
                                    self.scroll_offset = self.selected_listing_index;
                                }
                            }
                        }
                    }
                } else {
                    // If not locked, scroll the entire scrollview
                    self.scroll_view_state.scroll_up();
                }
            }
            KeyCode::PageDown => {
                if self.section_locked {
                    match self.scroll_view_mode {
                        ScrollViewMode::Paragraph => {
                            // Page down in paragraph view
                            self.paragraph_scroll_offset += 10;
                        }
                        ScrollViewMode::Table => {
                            if !self.listings.is_empty() {
                                let visible_rows = 25;
                                let new_selected = std::cmp::min(
                                    self.selected_listing_index + visible_rows,
                                    self.listings.len() - 1
                                );
                                self.selected_listing_index = new_selected;
                                
                                // Adjust scroll to keep selection visible
                                if self.selected_listing_index >= self.scroll_offset + visible_rows {
                                    self.scroll_offset = self.selected_listing_index - visible_rows + 1;
                                }
                            }
                        }
                    }
                } else {
                    self.scroll_view_state.scroll_page_down();
                }
            }
            KeyCode::PageUp => {
                if self.section_locked {
                    match self.scroll_view_mode {
                        ScrollViewMode::Paragraph => {
                            // Page up in paragraph view
                            self.paragraph_scroll_offset = self.paragraph_scroll_offset.saturating_sub(10);
                        }
                        ScrollViewMode::Table => {
                            if !self.listings.is_empty() {
                                let visible_rows = 25;
                                let new_selected = self.selected_listing_index.saturating_sub(visible_rows);
                                self.selected_listing_index = new_selected;
                                
                                // Adjust scroll to keep selection visible
                                if self.selected_listing_index < self.scroll_offset {
                                    self.scroll_offset = self.selected_listing_index;
                                }
                            }
                        }
                    }
                } else {
                    self.scroll_view_state.scroll_page_up();
                }
            }
            KeyCode::Home => {
                if self.section_locked {
                    match self.scroll_view_mode {
                        ScrollViewMode::Paragraph => {
                            // Go to top of paragraph
                            self.paragraph_scroll_offset = 0;
                        }
                        ScrollViewMode::Table => {
                            if !self.listings.is_empty() {
                                self.selected_listing_index = 0;
                                self.scroll_offset = 0;
                            }
                        }
                    }
                } else {
                    self.scroll_view_state.scroll_to_top();
                }
            }
            KeyCode::End => {
                if self.section_locked {
                    match self.scroll_view_mode {
                        ScrollViewMode::Paragraph => {
                            // Go to bottom of paragraph (approximate)
                            self.paragraph_scroll_offset = 50; // Adjust based on content
                        }
                        ScrollViewMode::Table => {
                            if !self.listings.is_empty() {
                                self.selected_listing_index = self.listings.len() - 1;
                                let visible_rows = 25;
                                self.scroll_offset = if self.listings.len() > visible_rows {
                                    self.listings.len() - visible_rows
                                } else {
                                    0
                                };
                            }
                        }
                    }
                } else {
                    self.scroll_view_state.scroll_to_bottom();
                }
            }
            KeyCode::Char('i') => {
                // Only works in table mode
                if self.scroll_view_mode == ScrollViewMode::Table &&
                   !self.listings.is_empty() && 
                   self.selected_listing_index < self.listings.len() {
                    if let Some(url) = &self.listings[self.selected_listing_index].url {
                        let _ = std::process::Command::new("firefox")
                            .arg(url)
                            .spawn();
                    }
                }
            }
            KeyCode::Tab => {
                // Switch between sections (only when not locked)
                if !self.section_locked {
                    match self.scroll_view_mode {
                        ScrollViewMode::Paragraph => self.scroll_view_mode = ScrollViewMode::Table,
                        ScrollViewMode::Table => self.scroll_view_mode = ScrollViewMode::Paragraph,
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Monitors the current page URL for CAPTCHA challenges and handles the scraping workflow.
    /// This runs asynchronously and triggers appropriate events when CAPTCHA is detected or resolved.
    pub async fn start_captcha_monitoring(&mut self) -> color_eyre::Result<()> {
        if let Some(client) = self.client.clone() {
            let sender = self.events.sender.clone();
            
            tokio::spawn(async move {
                let mut captcha_detected = false;
                
                loop {
                    if let Ok(current_url) = client.current_url().await {
                        let url_has_captcha = current_url.to_string().to_lowercase().contains("captcha");
                        
                        if url_has_captcha && !captcha_detected {
                            // First time detecting captcha
                            info!("🔍 CAPTCHA detected in URL: {}", current_url);
                            captcha_detected = true;
                            let _ = sender.send(Event::App(AppEvent::CaptchaDetected));
                        } else if !url_has_captcha && captcha_detected {
                            // CAPTCHA was resolved
                            info!("✅ CAPTCHA no longer detected - continuing");
                            let _ = sender.send(Event::App(AppEvent::CaptchaResolved));
                            break;
                        } else if !url_has_captcha && !captcha_detected {
                            // No captcha detected from the start - proceed immediately
                            let _ = sender.send(Event::App(AppEvent::CaptchaResolved));
                            break;
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    } else {
                        break;
                    }
                }
            });
        }
        Ok(())
    }

    /// Set running to false to quit the application.
    pub async fn quit(&mut self) -> color_eyre::Result<()> {
        if let Some(client) = self.client.take() {
            info!("Quitting fantoccini client");
            client.close().await?;
        }
        if let Some(mut child) = self.geckodriver.take() {
            info!("Killing geckodriver");
            child.kill().unwrap();
        }
        self.running = false;
        Ok(())
    }

    /// Connect to the webdriver client.
    pub async fn connect(&mut self) -> color_eyre::Result<()> {
        info!("Starting geckodriver");
        self.events.send(AppEvent::SetProgress(
            0.1,
            "🚀 Starting geckodriver...".to_string(),
        ));
        let child = Command::new("./geckodriver")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        self.geckodriver = Some(child);
        self.events.send(AppEvent::SetProgress(
            0.2,
            "🔗 Connecting to fantoccini...".to_string(),
        ));
        info!("Connecting to webdriver");
        match ClientBuilder::native()
            .connect("http://localhost:4444")
            .await
        {
            Ok(client) => {
                client.minimize_window().await?;
                self.client = Some(client);
                self.events.send(AppEvent::SetProgress(
                    0.3,
                    "🌐 Navigating to ebay.com...".to_string(),
                ));
                self.events.send(AppEvent::ClientReady);
                info!("Webdriver client connected");
            }
            Err(e) => {
                error!("Failed to connect to webdriver: {}", e);
                self.quit().await?;
            }
        }
        Ok(())
    }

    

    /// Navigates the WebDriver client to the specified eBay seller page.
    pub async fn navigate_to_public_page(&mut self, url: String) -> color_eyre::Result<()> {
        info!("Navigating to {}", url);
        if let Some(client) = &mut self.client {
            client.goto(&url).await?;
            info!("Navigated to {}", url);
        }
        Ok(())
    }

    /// Static version of scrape_items_sold for use in async tasks
    pub async fn scrape_items_sold_static(client: &Client) -> color_eyre::Result<u32> {
        info!("Attempting to scrape items sold");
        match client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(fantoccini::Locator::Css(
                "div[title*='items sold'] > span",
            ))
            .await
        {
            Ok(sold_items_element) => {
                let sold_items_text = sold_items_element.text().await?;
                info!("Raw sold items text: {}", sold_items_text);
                let sold_items_count =
                    sold_items_text.replace(",", "").parse::<u32>().unwrap_or_else(|e| {
                        error!("Failed to parse sold items count: {}", e);
                        0
                    });
                Ok(sold_items_count)
            }
            Err(e) => {
                info!("Could not find sold items element: {}", e);
                Ok(0)
            }
        }
    }

    /// Static version of scrape_feedback for use in async tasks
    pub async fn scrape_feedback_static(client: &Client) -> color_eyre::Result<String> {
        info!("Attempting to scrape feedback");
        match client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(fantoccini::Locator::Css(
                ".str-seller-card__feedback-link",
            ))
            .await
        {
            Ok(feedback_element) => {
                let feedback_text = feedback_element.text().await?;
                info!("Feedback text: {}", feedback_text);
                Ok(feedback_text)
            }
            Err(e) => {
                info!("Could not find feedback element: {}", e);
                Ok(String::new())
            }
        }
    }

    /// Static version of scrape_follower_count for use in async tasks
    pub async fn scrape_follower_count_static(client: &Client) -> color_eyre::Result<u32> {
        info!("Attempting to scrape follower count");
        match client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(fantoccini::Locator::Css(
                r#".str-seller-card__store-stats-content > div:nth-child(3)"#,
            ))
            .await
        {
            Ok(follower_element) => {
                let follower_text = follower_element.text().await?;
                info!("Raw follower text: {}", follower_text);
                // Extract just the numeric part from text like "1 follower" or "123 followers"
                let follower_count = follower_text
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .replace(",", "")
                    .parse::<u32>()
                    .unwrap_or_else(|e| {
                        error!("Failed to parse follower count: {}", e);
                        0
                    });
                Ok(follower_count)
            }
            Err(e) => {
                info!("Could not find follower element: {}", e);
                Ok(0)
            }
        }
    }

    /// Static version of click_see_all_button for use in async tasks
    pub async fn click_see_all_button_static(client: &Client) -> color_eyre::Result<()> {
        info!("Attempting to click the 'see all' button");
        match client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(fantoccini::Locator::Css(
                ".str-marginals__footer--button",
            ))
            .await
        {
            Ok(button) => {
                button.click().await?;
                info!("'See all' button clicked successfully");
                Ok(())
            }
            Err(e) => {
                info!("Could not find 'see all' button: {}", e);
                Err(e.into())
            }
        }
    }

    /// Scrapes eBay listings from HTML content and returns a vector of Listing structs.
    /// This function parses the provided HTML and extracts listing information suitable for CSV export.
    pub fn scrape_listings_from_html(html_content: &str) -> color_eyre::Result<Vec<Listing>> {
        let document = Html::parse_document(html_content);
        
        // Try multiple selectors to handle different eBay listing formats
        let possible_selectors = vec![
            "div.su-card-container",
            "div.s-item__wrapper", 
            "li.s-item",
            ".str-item-card",
            ".item-listing-cell",
            "[data-testid='item-card']",
            ".str-grid-item"
        ];
        
        let mut elements = Vec::new();
        let mut successful_selector = "";
        
        for selector_str in &possible_selectors {
            match Selector::parse(selector_str) {
                Ok(selector) => {
                    let found_elements: Vec<_> = document.select(&selector).collect();
                    if !found_elements.is_empty() {
                        elements = found_elements;
                        successful_selector = selector_str;
                        break;
                    }
                }
                Err(e) => {
                    error!("Invalid selector '{}': {}", selector_str, e);
                    continue;
                }
            }
        }

        if elements.is_empty() {
            info!("No listings found with any of the known selectors");
            return Ok(Vec::new());
        }
        
        info!("Found {} listings using selector: {}", elements.len(), successful_selector);
        let mut listings = Vec::new();

        for (index, element) in elements.into_iter().enumerate() {
            let mut listing = Listing::default();
            info!("Processing element #{} with selector: {}", index + 1, successful_selector);

            // Extract title - try multiple selectors based on format
            listing.title = Self::extract_text_from_selectors(&element, &[
                // New su-card format
                ".s-card__title .su-styled-text",
                "div[role='heading'] .su-styled-text",
                // Old s-item format
                "div.s-item__title span[role='heading']",
                ".s-item__title span",
                ".s-item__title",
                // Generic fallbacks
                "h3",
                ".title",
                "[role='heading']"
            ]).unwrap_or_default();

            // Extract price - try multiple selectors
            listing.price = Self::extract_text_from_selectors(&element, &[
                // New su-card format
                ".s-card__price",
                ".su-styled-text.primary.bold",
                // Old s-item format  
                "span.s-item__price",
                ".s-item__detail--primary .s-item__price",
                // Generic fallbacks
                ".price"
            ]).unwrap_or_default();

            // Extract shipping cost
            if let Some(shipping_text) = Self::extract_text_from_selectors(&element, &[
                // New format - look for delivery/shipping text
                ".s-card__attribute-row",
                ".su-styled-text",
                // Old format
                "span.s-item__shipping",
                ".s-item__logisticsCost",
            ]) {
                // Filter for text containing delivery or shipping info
                if shipping_text.to_lowercase().contains("delivery") || 
                   shipping_text.to_lowercase().contains("shipping") ||
                   shipping_text.contains("$") {
                    listing.shipping = Some(shipping_text);
                }
            }

            // Extract condition
            if let Some(condition_text) = Self::extract_text_from_selectors(&element, &[
                // New format
                ".s-card__subtitle .su-styled-text",
                // Old format
                "span.SECONDARY_INFO",
            ]) {
                listing.condition = Some(condition_text);
            }

            // Extract location
            if let Some(location_text) = Self::extract_text_from_selectors(&element, &[
                // New format - look for "Located in" text
                ".su-styled-text",
                // Old format
                ".s-item__location",
            ]) {
                // Filter for text containing location info
                if location_text.to_lowercase().contains("located") || 
                   location_text.to_lowercase().contains("from") {
                    listing.location = Some(location_text);
                }
            }

            // Extract seller information  
            if let Some(seller_text) = Self::extract_text_from_selectors(&element, &[
                // New format - seller name and feedback are separate
                ".su-card-container__attributes__secondary .su-styled-text",
                // Old format
                ".s-item__etrs-text .PRIMARY",
                ".s-item__seller-info-text",
            ]) {
                // Parse seller name and feedback from text like "thriftngo5 95.7% positive (21)"
                let parts: Vec<&str> = seller_text.split_whitespace().collect();
                if !parts.is_empty() {
                    listing.seller = Some(parts[0].to_string());
                    // Look for feedback percentage in the remaining text
                    let feedback_text = parts[1..].join(" ");
                    if feedback_text.contains('%') {
                        listing.seller_feedback = Some(feedback_text);
                    }
                }
            }

            // Check for "Best Offer" availability
            listing.accepts_offers = Self::text_contains(&element, &[
                ".su-styled-text",
                ".s-item__dynamic", 
                ".s-item__formatBestOfferEnabled"
            ], "best offer") || Self::text_contains(&element, &[
                ".su-styled-text",
                ".s-item__dynamic"
            ], "or best offer");

            // Extract item URL from href attributes to get item ID
            let link_selectors = vec!["a", ".su-link", ".s-item__link"];
            for link_selector in &link_selectors {
                if let Ok(selector) = Selector::parse(link_selector) {
                    if let Some(link_element) = element.select(&selector).next() {
                        if let Some(href) = link_element.value().attr("href") {
                            // Extract item ID from URL if possible
                            if let Some(item_id_match) = href.split("itm/").nth(1) {
                                if let Some(item_id) = item_id_match.split('?').next() {
                                    listing.item_id = Some(item_id.to_string());
                                    listing.url = Some(format!("https://www.ebay.com/itm/{}", item_id));
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Only add listings that have at least a title and price
            if !listing.title.is_empty() && !listing.price.is_empty() {
                info!("Adding valid listing #{}: {} - {}", index + 1, listing.title, listing.price);
                listings.push(listing);
            } else {
                info!("Skipping listing #{}: missing title or price (title: '{}', price: '{}')", 
                     index + 1, listing.title, listing.price);
            }
        }

        info!("Successfully scraped {} listings from HTML using selector: {}", listings.len(), successful_selector);
        Ok(listings)
    }

    /// Helper function to extract text from the first matching selector
    fn extract_text_from_selectors(element: &scraper::ElementRef, selectors: &[&str]) -> Option<String> {
        for &selector_str in selectors {
            match Selector::parse(selector_str) {
                Ok(selector) => {
                    if let Some(elem) = element.select(&selector).next() {
                        let text = elem.text().collect::<Vec<_>>().join(" ").trim().to_string();
                        if !text.is_empty() {
                            return Some(text);
                        }
                    }
                }
                Err(_) => continue,
            }
        }
        None
    }


    /// Helper function to check if text contains a specific substring
    fn text_contains(element: &scraper::ElementRef, selectors: &[&str], search_text: &str) -> bool {
        selectors.iter().any(|&selector_str| {
            if let Ok(selector) = Selector::parse(selector_str) {
                element.select(&selector).any(|elem| {
                    elem.text().collect::<Vec<_>>().join(" ").to_lowercase().contains(search_text)
                })
            } else {
                false
            }
        })
    }

    /// Scrapes active eBay listings from the current page using the WebDriver client.
    /// Returns a vector of structured Listing objects ready for CSV export.
    pub async fn scrape_active_listings(client: &Client) -> color_eyre::Result<Vec<Listing>> {
        info!("Starting to scrape active listings from current page");

        // Wait a bit for page content to load
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        // Try to wait for any potential listing elements to appear
        let wait_selectors = vec![
            "div.su-card-container",
            "div.s-item__wrapper",
            "li.s-item",
            ".str-item-card", 
            ".item-listing-cell",
            "[data-testid='item-card']",
            ".str-grid-item"
        ];
        
        for selector in &wait_selectors {
            if let Ok(_) = client
                .wait()
                .at_most(std::time::Duration::from_secs(5))
                .for_element(fantoccini::Locator::Css(selector))
                .await 
            {
                info!("Found elements with selector: {}", selector);
                break;
            }
        }

        // Get the page source HTML
        let page_source = client.source().await?;
        
        // Parse the HTML and extract listings
        let listings = Self::scrape_listings_from_html(&page_source)?;
        
        info!("Successfully scraped {} active listings", listings.len());
        Ok(listings)
    }

    /// Scrapes item specifics and description from an individual eBay item page.
    /// This function takes an item URL and extracts detailed information.
    pub async fn scrape_item_details(client: &Client, item_url: &str) -> color_eyre::Result<(Vec<String>, Option<String>)> {
        info!("Scraping item details from: {}", item_url);
        
        // Navigate to the item page
        client.goto(item_url).await?;
        
        // Wait for the page to load
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        let mut item_specifics = Vec::new();
        
        // Extract item specifics using WebDriver element finding
        // Look for condition
        if let Ok(condition_elem) = client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(fantoccini::Locator::Css("dl.ux-labels-values--condition .ux-labels-values__values span.ux-textspans"))
            .await
        {
            if let Ok(text) = condition_elem.text().await {
                item_specifics.push(format!("Condition: {}", text.trim()));
            }
        }
        
        // Look for brand
        if let Ok(brand_elem) = client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(fantoccini::Locator::Css("dl.ux-labels-values--brand .ux-labels-values__values span.ux-textspans"))
            .await
        {
            if let Ok(text) = brand_elem.text().await {
                item_specifics.push(format!("Brand: {}", text.trim()));
            }
        }
        
        // Look for model
        if let Ok(model_elem) = client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(fantoccini::Locator::Css("dl.ux-labels-values--model .ux-labels-values__values span.ux-textspans"))
            .await
        {
            if let Ok(text) = model_elem.text().await {
                item_specifics.push(format!("Model: {}", text.trim()));
            }
        }
        
        // Look for color
        if let Ok(color_elem) = client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(fantoccini::Locator::Css("dl.ux-labels-values--color .ux-labels-values__values span.ux-textspans"))
            .await
        {
            if let Ok(text) = color_elem.text().await {
                item_specifics.push(format!("Color: {}", text.trim()));
            }
        }
        
        // Try to get description (simplified approach)
        let description = if let Ok(desc_iframe) = client
            .wait()
            .at_most(std::time::Duration::from_secs(3))
            .for_element(fantoccini::Locator::Css("#desc_ifr"))
            .await
        {
            // Get iframe src and try to extract some basic info
            if let Ok(src) = desc_iframe.attr("src").await {
                Some(format!("Description iframe: {}", src.unwrap_or_default()))
            } else {
                None
            }
        } else {
            None
        };
        
        info!("Extracted {} item specifics", item_specifics.len());
        Ok((item_specifics, description))
    }
    
    
    /// Enhanced function to scrape listings and enrich them with detailed information.
    /// This visits each item page to get item specifics and descriptions.
    pub async fn scrape_listings_with_details(client: &Client) -> color_eyre::Result<Vec<Listing>> {
        info!("Starting to scrape listings with detailed information");
        
        // First get the basic listings
        let mut listings = Self::scrape_active_listings(client).await?;
        
        let total_listings = listings.len();
        info!("Enriching {} listings with detailed information", total_listings);
        
        // For each listing, scrape detailed information
        for (index, listing) in listings.iter_mut().enumerate() {
            info!("Processing listing {}/{}: {}", index + 1, total_listings, listing.title);
            
            // Construct URL from item_id
            if let Some(item_id) = &listing.item_id {
                let item_url = format!("https://www.ebay.com/itm/{}", item_id);
                
                match Self::scrape_item_details(client, &item_url).await {
                    Ok((item_specifics, description)) => {
                        listing.item_specifics = item_specifics;
                        listing.description = description;
                        info!("Successfully enriched listing: {}", listing.title);
                    }
                    Err(e) => {
                        error!("Failed to scrape details for {}: {}", listing.title, e);
                        // Continue with the next listing rather than failing completely
                    }
                }
                
                // Add a small delay between requests to be respectful
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
            }
        }
        
        info!("Completed enriching listings with details");
        Ok(listings)
    }

    /// Scrapes active eBay listings from the current page and saves them to a CSV file.
    /// This is a convenience method that combines scraping and CSV export.
    pub async fn scrape_and_save_to_csv(client: &Client, filename: &str) -> color_eyre::Result<()> {
        let listings = Self::scrape_active_listings(client).await?;
        
        if listings.is_empty() {
            info!("No listings found to save");
            return Ok(());
        }

        // Create CSV writer
        let mut wtr = csv::Writer::from_path(filename)?;
        
        // Write CSV headers
        wtr.write_record(&[
            "title", "price", "shipping", "condition", "watchers", "seller", 
            "seller_feedback", "buy_it_now", "accepts_offers", "location", 
            "quantity_available", "is_new_listing", "item_id", "url", "notes",
            "item_specifics", "description"
        ])?;

        // Write listing data
        for listing in &listings {
            let watchers_str = listing.watchers.map_or(String::new(), |w| w.to_string());
            let quantity_str = listing.quantity_available.map_or(String::new(), |q| q.to_string());
            let buy_it_now_str = listing.buy_it_now.to_string();
            let accepts_offers_str = listing.accepts_offers.to_string();
            let is_new_listing_str = listing.is_new_listing.to_string();
            let notes_str = listing.notes.join("; ");
            let item_specifics_str = listing.item_specifics.join("; ");
            
            wtr.write_record(&[
                &listing.title,
                &listing.price,
                listing.shipping.as_deref().unwrap_or(""),
                listing.condition.as_deref().unwrap_or(""),
                &watchers_str,
                listing.seller.as_deref().unwrap_or(""),
                listing.seller_feedback.as_deref().unwrap_or(""),
                &buy_it_now_str,
                &accepts_offers_str,
                listing.location.as_deref().unwrap_or(""),
                &quantity_str,
                &is_new_listing_str,
                listing.item_id.as_deref().unwrap_or(""),
                listing.url.as_deref().unwrap_or(""),
                &notes_str,
                &item_specifics_str,
                listing.description.as_deref().unwrap_or(""),
            ])?;
        }

        wtr.flush()?;
        info!("Successfully saved {} listings to {}", listings.len(), filename);
        Ok(())
    }
    
    /// Saves the currently stored listings to a CSV file.
    pub fn save_listings_to_csv(&self, filename: &str) -> color_eyre::Result<()> {
        if self.listings.is_empty() {
            info!("No listings to save");
            return Ok(());
        }

        let mut wtr = csv::Writer::from_path(filename)?;
        
        wtr.write_record(&[
            "title", "price", "shipping", "condition", "watchers", "seller", 
            "seller_feedback", "buy_it_now", "accepts_offers", "location", 
            "quantity_available", "is_new_listing", "item_id", "url", "notes",
            "item_specifics", "description"
        ])?;

        for listing in &self.listings {
            let watchers_str = listing.watchers.map_or(String::new(), |w| w.to_string());
            let quantity_str = listing.quantity_available.map_or(String::new(), |q| q.to_string());
            let buy_it_now_str = listing.buy_it_now.to_string();
            let accepts_offers_str = listing.accepts_offers.to_string();
            let is_new_listing_str = listing.is_new_listing.to_string();
            let notes_str = listing.notes.join("; ");
            let item_specifics_str = listing.item_specifics.join("; ");
            
            wtr.write_record(&[
                &listing.title,
                &listing.price,
                listing.shipping.as_deref().unwrap_or(""),
                listing.condition.as_deref().unwrap_or(""),
                &watchers_str,
                listing.seller.as_deref().unwrap_or(""),
                listing.seller_feedback.as_deref().unwrap_or(""),
                &buy_it_now_str,
                &accepts_offers_str,
                listing.location.as_deref().unwrap_or(""),
                &quantity_str,
                &is_new_listing_str,
                listing.item_id.as_deref().unwrap_or(""),
                listing.url.as_deref().unwrap_or(""),
                &notes_str,
                &item_specifics_str,
                listing.description.as_deref().unwrap_or(""),
            ])?;
        }

        wtr.flush()?;
        info!("Successfully saved {} listings to {}", self.listings.len(), filename);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_listings_from_html() {
        // Sample HTML content that mimics eBay listing structure
        let sample_html = r#"
        <div>
            <ul class="srp-results">
                <li class="s-item">
                    <h3 class="s-item__title">Sample Item Title</h3>
                    <span class="s-item__price">$19.99</span>
                    <span class="s-item__shipping">+$4.99 shipping</span>
                    <span class="SECONDARY_INFO">Used</span>
                    <span class="s-item__seller-info-text">seller123</span>
                    <span class="s-item__watchheart-count">5</span>
                    <div class="s-item__purchase-options-with-icon">Buy It Now</div>
                    <div class="s-item__trending">New Listing</div>
                    <span class="s-item__location">From United States</span>
                </li>
                <li class="s-item">
                    <h3 class="s-item__title">Another Sample Item</h3>
                    <span class="s-item__price">$29.99</span>
                    <span class="s-item__shipping">Free shipping</span>
                    <span class="SECONDARY_INFO">New</span>
                </li>
            </ul>
        </div>
        "#;

        let result = App::scrape_listings_from_html(sample_html);
        assert!(result.is_ok());
        
        let listings = result.unwrap();
        assert_eq!(listings.len(), 2);

        // Test first listing
        let first_listing = &listings[0];
        assert_eq!(first_listing.title, "Sample Item Title");
        assert_eq!(first_listing.price, "$19.99");
        assert_eq!(first_listing.shipping, Some("+$4.99 shipping".to_string()));
        assert_eq!(first_listing.condition, Some("Used".to_string()));
        assert_eq!(first_listing.seller, Some("seller123".to_string()));
        assert_eq!(first_listing.watchers, Some(5));
        assert!(first_listing.buy_it_now);
        assert!(first_listing.is_new_listing);
        assert_eq!(first_listing.location, Some("From United States".to_string()));

        // Test second listing
        let second_listing = &listings[1];
        assert_eq!(second_listing.title, "Another Sample Item");
        assert_eq!(second_listing.price, "$29.99");
        assert_eq!(second_listing.shipping, Some("Free shipping".to_string()));
        assert_eq!(second_listing.condition, Some("New".to_string()));
    }
}
