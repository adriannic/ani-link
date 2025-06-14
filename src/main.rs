use config::Config;
use gui::main_menu::main_menu;
use std::error::Error;

mod config;
mod gui;
mod scraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut config = Config::init()?;
    let mut term = ratatui::init();

    main_menu(&mut config, &mut term).await?;

    ratatui::restore();

    Ok(())
}
