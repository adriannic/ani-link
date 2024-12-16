use std::error::Error;

use async_trait::async_trait;

#[async_trait]
pub(crate) trait Scraper {
    async fn try_search(query: &str) -> Result<Vec<String>, Box<dyn Error>>;
    async fn try_get_episodes(anime: &str) -> Result<Vec<usize>, Box<dyn Error>>;
    async fn try_get_mirrors(anime: &str, episode: usize) -> Result<Vec<String>, Box<dyn Error>>;
}
