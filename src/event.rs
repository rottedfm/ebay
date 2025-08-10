use color_eyre::eyre::OptionExt;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use std::time::Duration;
use tokio::sync::mpsc;
use fantoccini::{Client, ClientBuilder};
use std::process::{Child, Command};

/// The frequency at which tick events are emitted.
const TICK_FPS: f64 = 30.0;

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
}

/// Application events.
///
/// You can extend this enum with your own custom events.
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Quit the application.
    Quit,
    /// Connect to the webdriver client.
    Connect,
    /// The client is ready.
    ClientReady,
    /// Set the loading progress.
    SetProgress(f64, String),
    /// Initialize the application.
    Init(String),
    /// Scrape the feedback score.
    ScrapeFeedback(String),
    /// Scrape the number of items sold.
    ScrapeItemsSold(u32),
    /// Scrape the follower count.
    ScrapeFollowerCount(u32),
    /// Geckodriver started successfully.
    GeckodriverStarted,
    /// Geckodriver failed to start.
    GeckodriverError(String),
    /// WebDriver client connected.
    WebDriverConnected,
    /// WebDriver client connection failed.
    WebDriverError(String),
    /// Navigate to a URL.
    NavigateToUrl(String),
    /// Navigation completed.
    NavigationComplete,
    /// Navigation failed.
    NavigationError(String),
    /// CAPTCHA detected on page.
    CaptchaDetected,
    /// User response to CAPTCHA prompt.
    CaptchaResponse(bool),
    /// Scraping operations completed.
    ScrapingComplete,
}

/// WebDriver handler for managing geckodriver and fantoccini client.
#[derive(Debug)]
pub struct WebDriverHandler {
    /// Geckodriver process handle.
    geckodriver_process: Option<Child>,
    /// Fantoccini client.
    client: Option<Client>,
    /// Event sender for async operations.
    sender: mpsc::UnboundedSender<Event>,
}

impl WebDriverHandler {
    /// Create a new WebDriver handler.
    pub fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            geckodriver_process: None,
            client: None,
            sender,
        }
    }

    /// Start geckodriver in a non-blocking way.
    pub async fn start_geckodriver(&mut self) -> color_eyre::Result<()> {
        let sender = self.sender.clone();
        
        tokio::spawn(async move {
            match Command::new("geckodriver")
                .args(["--port", "4444"])
                .spawn()
            {
                Ok(_child) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    let _ = sender.send(Event::App(AppEvent::GeckodriverStarted));
                }
                Err(e) => {
                    let _ = sender.send(Event::App(AppEvent::GeckodriverError(e.to_string())));
                }
            }
        });

        Ok(())
    }

    /// Connect to WebDriver in a non-blocking way.
    pub async fn connect_webdriver(&mut self) -> color_eyre::Result<()> {
        let sender = self.sender.clone();
        
        tokio::spawn(async move {
            match ClientBuilder::native()
                .connect("http://localhost:4444")
                .await
            {
                Ok(_client) => {
                    let _ = sender.send(Event::App(AppEvent::WebDriverConnected));
                }
                Err(e) => {
                    let _ = sender.send(Event::App(AppEvent::WebDriverError(e.to_string())));
                }
            }
        });

        Ok(())
    }

    /// Navigate to URL in a non-blocking way.
    pub async fn navigate_to_url(&self, url: String) -> color_eyre::Result<()> {
        if self.client.is_none() {
            return Ok(());
        }

        let sender = self.sender.clone();
        
        tokio::spawn(async move {
            match ClientBuilder::native()
                .connect("http://localhost:4444")
                .await
            {
                Ok(client) => {
                    match client.goto(&url).await {
                        Ok(_) => {
                            let _ = sender.send(Event::App(AppEvent::NavigationComplete));
                        }
                        Err(e) => {
                            let _ = sender.send(Event::App(AppEvent::NavigationError(e.to_string())));
                        }
                    }
                    let _ = client.close().await;
                }
                Err(e) => {
                    let _ = sender.send(Event::App(AppEvent::NavigationError(e.to_string())));
                }
            }
        });

        Ok(())
    }

    /// Clean up resources.
    pub fn cleanup(&mut self) {
        if let Some(mut process) = self.geckodriver_process.take() {
            let _ = process.kill();
        }
    }
}

impl Drop for WebDriverHandler {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    pub sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    /// WebDriver handler for async operations.
    webdriver_handler: WebDriverHandler,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });
        let webdriver_handler = WebDriverHandler::new(sender.clone());
        Self { sender, receiver, webdriver_handler }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the reciever cannot be dropped while this struct still has a
        // reference to it
        let _ = self.sender.send(Event::App(app_event));
    }

    /// Start geckodriver asynchronously.
    pub async fn start_geckodriver(&mut self) -> color_eyre::Result<()> {
        self.webdriver_handler.start_geckodriver().await
    }

    /// Connect to WebDriver asynchronously.
    pub async fn connect_webdriver(&mut self) -> color_eyre::Result<()> {
        self.webdriver_handler.connect_webdriver().await
    }

    /// Navigate to a URL asynchronously.
    pub async fn navigate_to_url(&mut self, url: String) -> color_eyre::Result<()> {
        self.webdriver_handler.navigate_to_url(url).await
    }

    /// Clean up WebDriver resources.
    pub fn cleanup_webdriver(&mut self) {
        self.webdriver_handler.cleanup();
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventTask {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    /// Constructs a new instance of [`EventThread`].
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    async fn run(self) -> color_eyre::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut reader = crossterm::event::EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);
        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
              _ = self.sender.closed() => {
                break;
              }
              _ = tick_delay => {
                self.send(Event::Tick);
              }
              Some(Ok(evt)) = crossterm_event => {
                self.send(Event::Crossterm(evt));
              }
            };
        }
        Ok(())
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}
