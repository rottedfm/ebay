use crate::app::App;
use crossterm::{execute, terminal::{Clear, ClearType}};
use std::io::stdout;


pub mod app;
pub mod event;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    clear_screen()?;
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().await?.run(terminal).await;
    ratatui::restore();
    result
}

fn clear_screen() -> std::io::Result<()> {
    execute!(stdout(), Clear(ClearType::All))?;
    Ok(())
}
