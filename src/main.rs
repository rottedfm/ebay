use crate::app::App;
use log::{error, info};
use std::io::IsTerminal;

pub mod app;
pub mod event;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let log_file = std::fs::File::create("app.log")?;
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();
    info!("Starting up");

    if !std::io::stdout().is_terminal() {
        error!("Not running in a TTY. Exiting.");
        return Ok(());
    }

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    if let Err(ref err) = result {
        error!("Error: {}", err);
    }
    info!("Shutting down");
    result
}
