use crate::{page::{AppUpdate, Page}, scraper::anime::Anime};

#[derive(Clone, Copy)]
pub enum PopupState {
    None,
    Mpv(f64),
    Syncplay(f64),
    Download(f64),
}

pub struct EpisodesPage {
    pub popup_state: PopupState,
    pub state: usize,
    pub anime_list: Vec<Anime>,
    pub anime: Anime,
    pub episodes: Vec<f64>,
}

impl Page for EpisodesPage {
    fn view(&self) -> iced::Element<'_, crate::app::Message> {
        todo!()
    }

    fn update(&mut self, message: crate::app::Message) -> AppUpdate {
        todo!()
    }

    fn subscription(&self) -> iced::Subscription<crate::app::Message> {
        todo!()
    }

    fn theme(&self) -> iced::Theme {
        todo!()
    }
}
