use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use reqwest::Client;
use std::error::Error;
use std::io::Stdout;
use std::process::Command;

use crate::config::Config;

const WHITELIST: [&str; 3] = ["mp4upload", "ok.ru", "my.mail.ru"];

pub async fn search(
    config: &Config,
    client: &Client,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    Command::new("notify-send")
        .args(["Ani-link", "search"])
        .output()
        .unwrap();
    Ok(())
}
