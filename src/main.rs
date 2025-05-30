use crate::main_menu::main_menu;
use config::Config;
use reqwest::Client;
use std::error::Error;

mod config;
mod main_menu;
mod scraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::init()?;

    println!("{:?}", config);

    let client = Client::new();
    let mut term = ratatui::init();

    main_menu(&config, &client, &mut term).await?;

    ratatui::restore();

    Ok(())
}
