use ratatui::widgets::ListState;

use crate::{app::search::SearchState, scraper::anime::Anime};

#[derive(Clone)]
pub enum MenuState {
    Episodes {
        anime: Anime,
    },
    MainMenu {
        searching: bool,
    },
    Options,
    Search {
        anime_list: Vec<Anime>,
        search_state: SearchState,
        query: String,
        anime_state: ListState,
        filtered_list: Vec<Anime>,
    },
}
