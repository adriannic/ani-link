use std::{error::Error, fmt};

use async_trait::async_trait;
use clap::ValueEnum;
use reqwest::Client;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Anime {
    pub url: String,
    pub name: String,
}

impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "PascalCase")]
pub enum ScraperImpl {
    AnimeFLVScraper,
    AnimeAV1Scraper,
}

#[async_trait]
pub(crate) trait Scraper {
    async fn try_search(client: &Client, query: &str) -> Result<Vec<Anime>, Box<dyn Error>>;
    async fn try_get_episodes(client: &Client, anime: &str) -> Result<Vec<usize>, Box<dyn Error>>;
    async fn try_get_mirrors(
        client: &Client,
        anime: &str,
        episode: usize,
    ) -> Result<Vec<String>, Box<dyn Error>>;
}
