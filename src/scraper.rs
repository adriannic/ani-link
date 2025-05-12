use std::error::Error;

use async_trait::async_trait;
use reqwest::Client;

#[async_trait]
pub(crate) trait Scraper {
    async fn try_search(client: &Client, query: &str) -> Result<Vec<String>, Box<dyn Error>>;
    async fn try_get_episodes(client: &Client, anime: &str) -> Result<Vec<usize>, Box<dyn Error>>;
    async fn try_get_mirrors(client: &Client, anime: &str, episode: usize) -> Result<Vec<String>, Box<dyn Error>>;
}
