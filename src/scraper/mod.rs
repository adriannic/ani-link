use std::{error::Error, fmt, str::FromStr};

use anime::Anime;
use clap::ValueEnum;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub mod anime;
pub mod animeav1scraper;
pub mod animeflvscraper;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseScraperError;

#[derive(
    ValueEnum, Clone, Debug, EnumIter, Copy, Serialize, Deserialize, PartialEq, Eq, Default,
)]
#[clap(rename_all = "PascalCase")]
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
    pub fn next(self) -> Self {
        match self {
            Self::AnimeAv1Scraper => Self::AnimeFlvScraper,
            Self::AnimeFlvScraper => Self::AnimeAv1Scraper,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::AnimeAv1Scraper => Self::AnimeFlvScraper,
            Self::AnimeFlvScraper => Self::AnimeAv1Scraper,
        }
    }
}

pub trait Scraper {
    fn try_search(client: &Client) -> Result<Vec<Anime>, Box<dyn Error>>;
    fn try_get_episodes(client: &Client, slug: &str) -> Result<Vec<f64>, Box<dyn Error>>;
    fn try_get_mirrors(
        client: &Client,
        slug: &str,
        episode: f64,
    ) -> Result<Vec<String>, Box<dyn Error>>;
}
