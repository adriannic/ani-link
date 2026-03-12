use std::{
    error::Error,
    fmt,
    str::FromStr,
    sync::{Arc, atomic::AtomicUsize},
};

use anime::Anime;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::scraper::{animeav1scraper::AnimeAv1Scraper, animeflvscraper::AnimeFlvScraper};

pub mod anime;
pub mod animeav1scraper;
pub mod animeflvscraper;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseScraperError;

#[derive(Clone, Debug, EnumIter, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ScraperImpl {
    #[default]
    AnimeAv1Scraper,
    AnimeFlvScraper,
}

impl FromStr for ScraperImpl {
    type Err = ParseScraperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AnimeAv1Scraper" => Ok(Self::AnimeAv1Scraper),
            "AnimeFlvScraper" => Ok(Self::AnimeFlvScraper),
            _ => Err(ParseScraperError {}),
        }
    }
}

impl fmt::Display for ScraperImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ScraperImpl {
    pub const fn next(self) -> Self {
        match self {
            Self::AnimeAv1Scraper => Self::AnimeFlvScraper,
            Self::AnimeFlvScraper => Self::AnimeAv1Scraper,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::AnimeAv1Scraper => Self::AnimeFlvScraper,
            Self::AnimeFlvScraper => Self::AnimeAv1Scraper,
        }
    }

    pub async fn try_search(
        &self,
        client: &Client,
        progress: Arc<AtomicUsize>,
    ) -> Result<Vec<Anime>, Box<dyn Error>> {
        match self {
            Self::AnimeAv1Scraper => AnimeAv1Scraper::try_search(client, progress).await,
            Self::AnimeFlvScraper => AnimeFlvScraper::try_search(client, progress).await,
        }
    }

    pub async fn try_get_episodes(
        &self,
        client: &Client,
        slug: &str,
    ) -> Result<Vec<f64>, Box<dyn Error>> {
        match self {
            Self::AnimeAv1Scraper => AnimeAv1Scraper::try_get_episodes(client, slug).await,
            Self::AnimeFlvScraper => AnimeFlvScraper::try_get_episodes(client, slug).await,
        }
    }

    pub async fn try_get_mirrors(
        &self,
        client: &Client,
        slug: &str,
        episode: f64,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        match self {
            Self::AnimeAv1Scraper => AnimeAv1Scraper::try_get_mirrors(client, slug, episode).await,
            Self::AnimeFlvScraper => AnimeFlvScraper::try_get_mirrors(client, slug, episode).await,
        }
    }

    pub fn pages(self) -> usize {
        match self {
            Self::AnimeAv1Scraper => AnimeAv1Scraper::pages(),
            Self::AnimeFlvScraper => AnimeFlvScraper::pages(),
        }
    }
}

pub trait Scraper {
    async fn try_search(
        client: &Client,
        progress: Arc<AtomicUsize>,
    ) -> Result<Vec<Anime>, Box<dyn Error>>;
    async fn try_get_episodes(client: &Client, slug: &str) -> Result<Vec<f64>, Box<dyn Error>>;
    async fn try_get_mirrors(
        client: &Client,
        slug: &str,
        episode: f64,
    ) -> Result<Vec<String>, Box<dyn Error>>;
    fn pages() -> usize;
}
