use std::{error::Error, fmt};

use anime::Anime;
use async_trait::async_trait;
use clap::ValueEnum;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

mod anime;
mod animeav1scraper;
mod animeflvscraper;

#[derive(ValueEnum, Clone, Debug, EnumIter, Copy, Serialize, Deserialize)]
#[clap(rename_all = "PascalCase")]
pub enum ScraperImpl {
    AnimeAv1Scraper,
    AnimeFlvScraper,
}

impl fmt::Display for ScraperImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ScraperImpl {
    pub fn previous(self) -> Self {
        match self {
            Self::AnimeAv1Scraper => Self::AnimeFlvScraper,
            Self::AnimeFlvScraper => Self::AnimeAv1Scraper,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::AnimeAv1Scraper => Self::AnimeFlvScraper,
            Self::AnimeFlvScraper => Self::AnimeAv1Scraper,
        }
    }
}

#[async_trait]
pub trait Scraper {
    async fn try_search(client: &Client, query: &str) -> Result<Vec<Anime>, Box<dyn Error>>;
    async fn try_get_episodes(client: &Client, anime: &str) -> Result<Vec<usize>, Box<dyn Error>>;
    async fn try_get_mirrors(
        client: &Client,
        anime: &str,
        episode: usize,
    ) -> Result<Vec<String>, Box<dyn Error>>;
}
