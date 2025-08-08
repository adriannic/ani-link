use std::{fmt, mem};

use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListDirection, ListState},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    app::App,
    config::Config,
    menu_state::{ListQueryState, MenuState},
};

#[derive(EnumIter)]
enum Options {
    Scraper,
    Pages,
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Scraper => "Scraper",
                Self::Pages => "Páginas",
            }
        )
    }
}

pub fn draw_options(frame: &mut Frame, content_area: Rect, config: &Config, state: &mut ListState) {
    // Render right block
    let options_title = Line::from(" Opciones ".bold().white()).centered();

    let options_instructions = Line::from(vec![
        " Subir:".white(),
        " ↑ K ".blue().bold(),
        " Bajar:".white(),
        " ↓ J ".blue().bold(),
        " Siguiente:".white(),
        " → L ".blue().bold(),
        " Anterior:".white(),
        " ← H ".blue().bold(),
        " Guardar:".white(),
        " Enter ".blue().bold(),
        " Salir sin guardar:".white(),
        " Esc Q ".blue().bold(),
    ]);

    let right_block = Block::bordered()
        .title(options_title)
        .title_bottom(options_instructions.centered())
        .border_set(border::THICK)
        .border_style(Style::new().green());

    let option_list = List::new(Options::iter().map(|option| {
        format!(
            "{}: {}",
            option,
            match option {
                Options::Scraper => config.scraper.to_string(),
                Options::Pages => config.pages.to_string(),
            }
        )
    }))
    .block(right_block)
    .highlight_symbol("> ")
    .highlight_style(Style::new().bold())
    .repeat_highlight_symbol(true)
    .direction(ListDirection::TopToBottom);

    frame.render_stateful_widget(option_list, content_area, state);
}

pub fn handle_events_options(app: &mut App) {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = event::read().expect("Couldn't read event from options menu")
    {
        let MenuState::Options {
            anime_list,
            state,
            old_config,
        } = &mut app.menu_state
        else {
            panic!("Invalid app state in options menu")
        };

        match code {
            KeyCode::Char('k') | KeyCode::Up => state.select_previous(),
            KeyCode::Char('j') | KeyCode::Down => state.select_next(),
            KeyCode::Char('l') | KeyCode::Right => {
                if let Some(i) = state.selected() {
                    let option = Options::iter().nth(i).unwrap();
                    match option {
                        Options::Scraper => app.config.scraper = app.config.scraper.next(),
                        Options::Pages => app.config.pages = (app.config.pages + 1).clamp(1, 50),
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(i) = state.selected() {
                    let option = Options::iter().nth(i).unwrap();
                    match option {
                        Options::Scraper => app.config.scraper = app.config.scraper.previous(),
                        Options::Pages => app.config.pages = (app.config.pages - 1).clamp(1, 50),
                    }
                }
            }
            KeyCode::Enter => {
                app.config.save().expect("Couldn't save config to file");

                app.menu_state = if app.config.scraper != old_config.scraper {
                    MenuState::MainMenu {
                        anime_list: ListQueryState::spawn(app.config.scraper, app.client.clone()),
                        should_draw_popup: false,
                    }
                } else {
                    MenuState::MainMenu {
                        anime_list: mem::take(anime_list),
                        should_draw_popup: false,
                    }
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.config = old_config.clone();
                app.menu_state = MenuState::MainMenu {
                    anime_list: mem::take(anime_list),
                    should_draw_popup: false,
                };
            }
            _ => {}
        }
    }
}
