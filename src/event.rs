use color_eyre::eyre::OptionExt;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use std::time::Duration;
use tokio::sync::mpsc;

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

/// Application-specific events for the eBay scraper.
/// These events coordinate the scraping workflow and WebDriver interactions.
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
    /// Click the see all button.
    ClickSeeAll,
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
    /// CAPTCHA has been resolved by user.
    CaptchaResolved,
    /// Scraping operations completed.
    ScrapingComplete,
    /// Scrape listings from current page.
    ScrapeListings(Vec<crate::app::Listing>),
    /// Enrich listings with detailed information.
    EnrichListings,
    /// Enriched listings ready for saving.
    EnrichedListings(Vec<crate::app::Listing>),
}


/// Central event handler that coordinates terminal events, app events, and WebDriver operations.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    pub sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Creates a new event handler and spawns a background task to process terminal events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });
        Self { sender, receiver }
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

    /// Sends an application event to be processed in the next event loop iteration.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore send errors - receiver only drops when app shuts down
        let _ = self.sender.send(Event::App(app_event));
    }

}

/// Background task that processes terminal input events and emits regular tick events for the UI.
struct EventTask {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    /// Creates a new event processing task.
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    /// Main event loop that processes terminal events and emits tick events at 30 FPS.
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

    /// Sends an event through the channel, ignoring errors on app shutdown.
    fn send(&self, event: Event) {
        // Ignore send errors - receiver drops during shutdown
        let _ = self.sender.send(event);
    }
}
