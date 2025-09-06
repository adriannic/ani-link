use iced::widget::{column, pick_list};
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    app::{App, AppEvent},
    scraper::ScraperImpl,
};

#[derive(Debug, Clone, Copy)]
pub enum OptionEvent {
    UpdateScraper(ScraperImpl),
}

#[derive(EnumIter)]
pub enum Options {
    Scraper,
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Scraper => "Scraper",
            }
        )
    }
}

pub fn draw_options<'a>(app: &'a App) -> iced::Element<'a, AppEvent> {
    column![pick_list(
        ScraperImpl::iter()
            .map(|scraper| scraper.to_string())
            .collect::<Vec<_>>(),
        Some(app.config.scraper.to_string()),
        |_| { AppEvent::Options(OptionEvent::UpdateScraper(app.config.scraper.next())) }
    )].into()
}

pub fn handle_events_options(app: &mut App) {
    // if let Event::Key(KeyEvent {
    //     code,
    //     kind: KeyEventKind::Press,
    //     ..
    // }) = event::read().expect("Couldn't read event from options menu")
    // {
    //     let MenuState::Options {
    //         anime_list,
    //         state,
    //         old_config,
    //     } = &mut app.menu_state
    //     else {
    //         panic!("Invalid app state in options menu")
    //     };
    //
    //     match code {
    //         KeyCode::Char('k') | KeyCode::Up => state.select_previous(),
    //         KeyCode::Char('j') | KeyCode::Down => state.select_next(),
    //         KeyCode::Char('l') | KeyCode::Right => {
    //             if let Some(i) = state.selected() {
    //                 let option = Options::iter().nth(i).unwrap();
    //                 match option {
    //                     Options::Scraper => app.config.scraper = app.config.scraper.next(),
    //                     Options::Pages => app.config.pages = (app.config.pages + 1).clamp(1, 50),
    //                 }
    //             }
    //         }
    //         KeyCode::Char('h') | KeyCode::Left => {
    //             if let Some(i) = state.selected() {
    //                 let option = Options::iter().nth(i).unwrap();
    //                 match option {
    //                     Options::Scraper => app.config.scraper = app.config.scraper.previous(),
    //                     Options::Pages => app.config.pages = (app.config.pages - 1).clamp(1, 50),
    //                 }
    //             }
    //         }
    //         KeyCode::Enter => {
    //             app.config.save().expect("Couldn't save config to file");
    //
    //             app.menu_state = if app.config.scraper != old_config.scraper {
    //                 MenuState::MainMenu {
    //                     anime_list: ListQueryState::spawn(app.config.scraper, app.client.clone()),
    //                     should_draw_popup: false,
    //                 }
    //             } else {
    //                 MenuState::MainMenu {
    //                     anime_list: mem::take(anime_list),
    //                     should_draw_popup: false,
    //                 }
    //             }
    //         }
    //         KeyCode::Esc | KeyCode::Char('q') => {
    //             app.config = old_config.clone();
    //             app.menu_state = MenuState::MainMenu {
    //                 anime_list: mem::take(anime_list),
    //                 should_draw_popup: false,
    //             };
    //         }
    //         _ => {}
    //     }
    // }
}
