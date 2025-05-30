use std::process::Command;

use std::error::Error;

use std::io::Stdout;

use ratatui::prelude::CrosstermBackend;

use ratatui::Terminal;

use reqwest::Client;

use crate::config::Config;

pub async fn options(
    config: &Config,
    client: &Client,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    Command::new("notify-send")
        .args(["Ani-link", "change_backend"])
        .output()
        .unwrap();
    Ok(())
}
