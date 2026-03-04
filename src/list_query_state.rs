use std::thread::{self, JoinHandle};

use reqwest::blocking::Client;

use crate::scraper::{
    Scraper, ScraperImpl, anime::Anime, animeav1scraper::AnimeAv1Scraper,
    animeflvscraper::AnimeFlvScraper,
};

pub enum ListQueryState {
    Obtaining(JoinHandle<Vec<Anime>>),
    Obtained(Vec<Anime>),
}

impl Default for ListQueryState {
    fn default() -> Self {
        Self::Obtained(vec![])
    }
}

impl ListQueryState {
    pub fn spawn(scraper: ScraperImpl, client: Client) -> Self {
        ListQueryState::Obtaining(thread::spawn(move || match scraper {
            ScraperImpl::AnimeAv1Scraper => {
                AnimeAv1Scraper::try_search(&client).expect("Couldn't retrieve the list of animes")
            }
            ScraperImpl::AnimeFlvScraper => {
                AnimeFlvScraper::try_search(&client).expect("Couldn't retrieve the list of animes")
            }
        }))
    }

    pub fn get(self) -> Self {
        match self {
            Self::Obtaining(handle) => {
                Self::Obtained(handle.join().expect("Thread couldn't be joined"))
            }
            Self::Obtained(..) => self,
        }
    }
}
