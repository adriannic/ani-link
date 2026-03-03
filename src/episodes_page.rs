use crate::{scraper::anime::Anime};

#[derive(Clone, Copy)]
pub enum PopupState {
    None,
    Mpv(f64),
    Syncplay(f64),
    Download(f64),
}

pub struct EpisodesPage {
    popup_state: PopupState,
    state: u64,
    anime_list: Vec<Anime>,
    anime: Anime,
    episodes: Vec<f64>,
}

