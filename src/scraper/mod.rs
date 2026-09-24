use std::{
    error::Error,
    sync::{Arc, atomic::AtomicUsize},
};

use anime::Anime;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

use crate::scraper::animeav1scraper::AnimeAv1Scraper;

pub mod anime;
pub mod animeav1scraper;

#[derive(
    Clone,
    Debug,
    EnumIter,
    EnumString,
    Display,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
)]
pub enum ScraperImpl {
    #[default]
    AnimeAv1Scraper,
}

impl ScraperImpl {
    pub const fn next(self) -> Self {
        match self {
            Self::AnimeAv1Scraper => Self::AnimeAv1Scraper,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::AnimeAv1Scraper => Self::AnimeAv1Scraper,
        }
    }

    pub async fn try_search(
        &self,
        client: &Client,
        progress: Arc<AtomicUsize>,
    ) -> Result<Vec<Anime>, Box<dyn Error>> {
        match self {
            Self::AnimeAv1Scraper => AnimeAv1Scraper::try_search(client, progress).await,
        }
    }

    pub async fn try_get_episodes(
        &self,
        client: &Client,
        slug: &str,
    ) -> Result<Vec<f64>, Box<dyn Error>> {
        match self {
            Self::AnimeAv1Scraper => AnimeAv1Scraper::try_get_episodes(client, slug).await,
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
        }
    }

    pub fn pages(self) -> usize {
        match self {
            Self::AnimeAv1Scraper => AnimeAv1Scraper::pages(),
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
