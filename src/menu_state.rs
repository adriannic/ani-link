use std::
    thread::{self, JoinHandle}
;

use ratatui::widgets::ListState;
use reqwest::blocking::Client;

use crate::{
    app::search::SearchState,
    config::Config,
    scraper::{
        Scraper, ScraperImpl, anime::Anime, animeav1scraper::AnimeAv1Scraper,
        animeflvscraper::AnimeFlvScraper,
    },
};

#[derive(Default)]
pub enum ListQueryState {
    #[default]
    Transitioning,
    Obtaining(JoinHandle<Vec<Anime>>),
    Obtained(Vec<Anime>),
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
            Self::Transitioning => panic!("Called get() method on Transitioning state"),
            Self::Obtaining(handle) => {
                Self::Obtained(handle.join().expect("Thread couldn't be joined"))
            }
            Self::Obtained(..) => self,
        }
    }
}

pub enum MenuState {
    Episodes {
        anime: Anime,
    },
    MainMenu {
        anime_list: ListQueryState,
        should_draw_popup: bool,
    },
    Options {
        anime_list: ListQueryState,
        old_config: Config,
        state: ListState,
    },
    Search {
        anime_list: Vec<Anime>,
        search_state: SearchState,
        query: String,
        anime_state: ListState,
        filtered_list: Vec<Anime>,
    },
}
