use crate::event::{AppEvent, Event, EventHandler};
use fantoccini::{Client, ClientBuilder};
use log::{error, info};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    DefaultTerminal,
};
use std::process::{Child, Command};
use tokio;

/// Application state.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
    /// The application is loading.
    #[default]
    Loading,
    /// The application is running.
    Running,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// The application state.
    pub state: AppState,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
    /// The fantoccini client.
    pub client: Option<Client>,
    /// The geckodriver process.
    pub geckodriver: Option<Child>,
    /// The loading progress.
    pub progress: f64,
    pub progress_message: String,
    pub feedback_score: Option<String>,
    pub items_sold: Option<u32>,
    pub follower_count: Option<u32>,
    pub captcha_detected: bool,
    pub waiting_for_user_input: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            state: AppState::default(),
            events: EventHandler::new(),
            counter: 0,
            client: None,
            geckodriver: None,
            progress: 0.0,
            progress_message: String::new(),
            feedback_score: None,
            items_sold: None,
            follower_count: None,
            captcha_detected: false,
            waiting_for_user_input: false,
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
                                1.0,
                                "✅ Scraping complete!".to_string(),
                            )));
                            
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            
                            let _ = sender.send(Event::App(AppEvent::ScrapingComplete));
                        });
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
                        self.captcha_detected = true;
                        self.waiting_for_user_input = true;
                        self.progress_message = "CAPTCHA detected! Press 'y' to continue or 'n' to skip.".to_string();
                    }
                    AppEvent::CaptchaResponse(continue_scraping) => {
                        self.captcha_detected = false;
                        self.waiting_for_user_input = false;
                        if continue_scraping {
                            self.progress_message = "Continuing with scraping...".to_string();
                        } else {
                            self.progress_message = "Scraping cancelled due to CAPTCHA.".to_string();
                        }
                    }
                    AppEvent::ScrapingComplete => {
                        self.state = AppState::Running;
                    }
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.waiting_for_user_input && self.captcha_detected {
            match key_event.code {
                KeyCode::Char('y' | 'Y') => {
                    self.events.send(AppEvent::CaptchaResponse(true));
                }
                KeyCode::Char('n' | 'N') => {
                    self.events.send(AppEvent::CaptchaResponse(false));
                }
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                // Other handlers you could add here.
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

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

    /// Scrape the number of items sold
    pub async fn scrape_items_sold(&mut self) -> color_eyre::Result<u32> {
        info!("Attempting to scrape items sold");
        if let Some(client) = &self.client {
            match client
                .wait()
                .for_element(fantoccini::Locator::Css(
                    "div[title*='items sold'] > span",
                ))
                .await
            {
                Ok(sold_items_element) => {
                    let sold_items_text = sold_items_element.text().await?;
                    info!("Raw sold items text: {}", sold_items_text); // Log the raw text
                    let sold_items_count =
                        sold_items_text.replace(",", "").parse::<u32>().unwrap_or_else(|e| {
                            error!("Failed to parse sold items count: {}", e); // Log parsing errors
                            0
                        });
                    Ok(sold_items_count)
                }
                Err(e) => {
                    info!("Could not find sold items element: {}", e);
                    Ok(0)
                }
            }
        } else {
            Ok(0)
        }
    }

    /// Scrape the positive feedback score.
    pub async fn scrape_feedback(&mut self) -> color_eyre::Result<String> {
        info!("Attempting to scrape feedback");
        if let Some(client) = &self.client {
            match client
                .wait()
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
        } else {
            Ok(String::new()) // Return an empty string if client is not available
        }
    }

    /// Scrape the follower count.
    pub async fn scrape_follower_count(&mut self) -> color_eyre::Result<u32> {
        info!("Attempting to scrape follower count");
        if let Some(client) = &self.client {
            match client
                .wait()
                .for_element(fantoccini::Locator::Css(
                    r#"div[title*="follower"] span.str-text-span.BOLD"#,
                ))
                .await
            {
                Ok(follower_element) => {
                    let follower_text = follower_element.text().await?;
                    info!("Raw follower text: {}", follower_text);
                    let follower_count =
                        follower_text.replace(",", "").parse::<u32>().unwrap_or_else(|e| {
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
        } else {
            Ok(0)
        }
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

    /// Check if URL contains captcha
    pub fn check_url_for_captcha(&mut self, url: &str) {
        if url.to_lowercase().contains("captcha") {
            self.events.send(AppEvent::SetProgress(
                self.progress,
                "⚠️  CAPTCHA detected in URL! Please solve it manually and press any key to continue...".to_string(),
            ));
        }
    }

    /// Find the public ebay page
    pub async fn navigate_to_public_page(&mut self, url: String) -> color_eyre::Result<()> {
        info!("Navigating to {}", url);
        self.check_url_for_captcha(&url);
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
            .for_element(fantoccini::Locator::Css(
                r#"div[title*="follower"] span.str-text-span.BOLD"#,
            ))
            .await
        {
            Ok(follower_element) => {
                let follower_text = follower_element.text().await?;
                info!("Raw follower text: {}", follower_text);
                let follower_count =
                    follower_text.replace(",", "").parse::<u32>().unwrap_or_else(|e| {
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
}
