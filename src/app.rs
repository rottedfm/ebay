use crate::event::{AppEvent, Event, EventHandler};
use color_eyre::eyre::eyre;
use fantoccini::{Client as FantocciniClient, ClientBuilder, Locator};
use std::process::{ Command, Stdio};
use tokio::time::{sleep, Duration, Instant};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub store_name: String,
    pub my_listings: Vec<Listing>,
    pub competitor_listings: Vec<Listing>,
    pub items_sold: String,
    pub followers: String,
    pub feedback: String,
    pub events: EventHandler,
    pub captcha: bool,
    pub client: Option<FantocciniClient>,
    pub last_fetch_time: Instant,
}


#[derive(Debug)]
pub struct Listing {
    pub image: String, // change to path
    pub title: String,
    pub price: String,
    pub shipping: String,
    pub status: String,
    pub sold_at_retail: Option<bool>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            my_listings: Vec::new(),
            store_name: String::new(),
            items_sold: String::new(),
            followers: String::new(),
            competitor_listings: Vec::new(),
            feedback: String::new(),
            events: EventHandler::new(),
            captcha: false,
            client: None,
            last_fetch_time: Instant::now(),
        }
    }
}

impl App {
    /// Contructs a new instance of [`App`].
    pub async fn new() -> color_eyre::Result<Self> {
        let mut app = Self::default();

        // Check for geckodriver in PATH
        let geckodriver_path = which::which("geckodriver")?;

        let _driver = Command::new(geckodriver_path)
            .arg("--port")
            .arg("4444")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        sleep(Duration::from_secs(2)).await;

        let client = ClientBuilder::native()
            .connect("http://localhost:4444")
            .await?;

        app.client = Some(client);

        // TODO: Fetch from config instead
        app.fetch_feedback().await?;

        // TODO: Fetch from config instead
        app.fetch_sold_items().await?;

        // TODO: Fetch from config instead
        app.fetch_followers().await?;


        Ok(app)
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick().await,
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => {
                        self.quit();
                        self.shutdown().await?;
                    },
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
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub async fn tick(&mut self) {
        if self.last_fetch_time.elapsed() >= Duration::from_secs(180) {
            self.last_fetch_time = Instant::now();

            if let Err(e) = self.fetch_feedback().await {
                eprintln!("Failed to fetch feedback: {:?}", e);
            }

            if let Err(e) = self.fetch_sold_items().await {
                eprintln!("Failed to fetch sold items: {:?}", e);
            }
            if let Err(e) = self.fetch_followers().await {
                eprintln!("Failed to fetch follower count: {:?}", e);
            }
        }
    }

    pub async fn shutdown(&mut self) -> color_eyre::Result<()> {
        let client = self.client.as_mut().ok_or_else(| | eyre!("No active Fantoccini client"))?;
        client.close_window().await?;
        Ok(())
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub async fn fetch_feedback(&mut self) -> color_eyre::Result<String> {
        // Check for active client
        let client = self.client.as_mut().ok_or_else(| | eyre!("No active Fantoccini client"))?;

        // fetch store page
        client.goto("https://www.ebay.com/usr/thriftngo5").await?;

        // wait for feedback
        let span = client
            .wait()
            .for_element(Locator::Css(".str-seller-card__store-stats-content > div:nth-child(1)"))
            .await
            .map_err(|e| eyre!("Couldn't find feedback span: {e}"))?;


        let percent_text = span.text().await?; // e.g., "100%"

        // Store it in app state
        self.feedback = percent_text.clone();

        Ok(percent_text)
    }

    pub async fn fetch_followers(&mut self) -> color_eyre::Result<String> {
        self.goto("https://www.ebay.com/usr/thriftngo5").await?;

        let client = self.client.as_mut().ok_or_else(| | eyre!("No active Fantoccini client"))?;

        // wait for feedback
        let span = client
            .wait()
            .for_element(Locator::Css(".str-seller-card__store-stats-content > div:nth-child(3)"))
            .await
            .map_err(|e| eyre!("Couldn't find feedback span: {e}"))?;

        let followers_text = span.text().await?; // e.g., "100%"

        // Store it in app state
        self.followers = followers_text.clone();

        Ok(followers_text)


    }

    pub async fn fetch_sold_items(&mut self) -> color_eyre::Result<String> {
        self.goto("https://www.ebay.com/usr/thriftngo5").await?;

        // Check for active client
        let client = self.client.as_mut().ok_or_else(| | eyre!("No active Fantoccini client"))?;


        // wait for feedback
        let span = client
            .wait()
            .for_element(Locator::Css(".str-seller-card__store-stats-content > div:nth-child(2)"))
            .await
            .map_err(|e| eyre!("Couldn't find feedback span: {e}"))?;


        let sold_text = span.text().await?; // e.g., "100%"

        // Store it in app state
        self.items_sold = sold_text.clone();

        Ok(sold_text)
    }

    pub async fn goto(&mut self, url: &str) -> color_eyre::Result<()> {

        let client = self.client.as_mut().ok_or_else(|| eyre!("No active Fantoccini client"))?;

        client.goto(url).await?;
        // Check if current URL contains "captcha"
        let mut current_url = client.current_url().await?.to_string();

        if current_url.to_ascii_lowercase().contains("captcha") {
            self.captcha = true;
            loop {
                let _ = sleep(Duration::from_secs(2));
                let new_url = client.current_url().await?.to_string();
                if new_url != current_url && !new_url.to_ascii_lowercase().contains("captcha") {
                    self.captcha = false;
                    break;
                }

                current_url = new_url;
            }

        } else {
            self.captcha = false;
        }

       Ok(())
    }
}
