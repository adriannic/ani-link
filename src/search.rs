use std::mem;

use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListDirection, ListState, Paragraph},
};
use rayon::prelude::*;
use rust_fuzzy_search::fuzzy_search;

use crate::{
    menu_state::{ListQueryState, MenuState},
    scraper::{
        Scraper, ScraperImpl, anime::Anime, animeav1scraper::AnimeAv1Scraper,
        animeflvscraper::AnimeFlvScraper,
    },
};

use crate::app::App;

const MAX_QUERY: usize = 80;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SearchState {
    Searching,
    Choosing,
}

pub fn draw_search(
    frame: &mut Frame,
    content_area: Rect,
    search_state: SearchState,
    query: &str,
    anime_state: &mut ListState,
    filtered_list: &[Anime],
) {
    let vertical = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]);

    let [search_area, list_area] = vertical.areas(content_area);

    // Render search block
    let search_title = Line::from(" Buscar ".bold().white()).centered();

    let search_instructions = Line::from(vec![
        " Atrás:".white(),
        " Esc ".blue().bold(),
        " Confirmar:".white(),
        " Enter ".blue().bold(),
    ]);

    let mut search_block = Block::bordered()
        .title(search_title)
        .border_set(border::THICK)
        .border_style(Style::new().green());

    if search_state == SearchState::Searching {
        search_block = search_block.title_bottom(search_instructions.centered());
    }

    let mut search_text = Paragraph::new(format!(" Nombre: {query}"))
        .block(search_block)
        .bold();

    if search_state == SearchState::Choosing {
        search_text = search_text.style(Style::new().gray());
    }

    frame.render_widget(search_text, search_area);

    // Render list block
    let list_title = Line::from(" Animes ".bold().white()).centered();

    let list_instructions = Line::from(vec![
        " Subir:".white(),
        " ↑ K ".blue().bold(),
        " Bajar:".white(),
        " ↓ J ".blue().bold(),
        " Confirmar:".white(),
        " → L Enter ".blue().bold(),
        " Atrás:".white(),
        " ← H Esc ".blue().bold(),
    ]);

    let mut list_block = Block::bordered()
        .title(list_title)
        .border_set(border::THICK)
        .border_style(Style::new().green());

    if search_state == SearchState::Choosing {
        list_block = list_block.title_bottom(list_instructions.centered());
    }

    let filtered_anime = filtered_list
        .iter()
        .map(|anime| anime.names[0].as_ref())
        .collect::<Vec<&str>>();

    let mut anime_list = List::new(filtered_anime)
        .block(list_block)
        .highlight_symbol("> ")
        .highlight_style(Style::new().bold())
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    if search_state == SearchState::Searching {
        anime_list = anime_list.style(Style::new().gray());
    }

    frame.render_stateful_widget(anime_list, list_area, anime_state);
}

pub fn handle_events_search(app: &mut App) {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = event::read().expect("Couldn't read event from search menu")
    {
        let MenuState::Search {
            search_state,
            query,
            anime_state,
            filtered_list,
            anime_list,
        } = &mut app.menu_state
        else {
            panic!("Invalid app state in search menu");
        };

        match search_state {
            SearchState::Searching => match code {
                KeyCode::Esc => {
                    app.menu_state = MenuState::MainMenu {
                        anime_list: ListQueryState::Obtained(mem::take(anime_list)),
                        should_draw_popup: false,
                    };
                }
                KeyCode::Enter => *search_state = SearchState::Choosing,
                KeyCode::Backspace => {
                    if !query.is_empty() {
                        anime_state.select_first();
                        query.pop();
                        if query.is_empty() {
                            anime_state.select(None);
                        }

                        let mut filtered_anime = fuzzy_search(
                            &query.to_lowercase(),
                            &anime_list
                                .par_iter()
                                .map(|anime| anime.names[0].to_lowercase().to_string())
                                .collect::<Vec<_>>()
                                .par_iter()
                                .map(String::as_ref)
                                .collect::<Vec<_>>(),
                        )
                        .par_iter()
                        .map(|(_, score)| *score)
                        .zip(anime_list.clone())
                        .collect::<Vec<_>>();

                        filtered_anime.par_sort_unstable_by(|a, b| {
                            b.0.partial_cmp(&a.0)
                                .expect("Comparison of f32 failed when sorting animes")
                        });

                        *filtered_list = filtered_anime
                            .into_iter()
                            .map(|(_, anime)| anime)
                            .collect::<Vec<_>>();
                    }
                }
                KeyCode::Char(c) => {
                    if query.len() < MAX_QUERY {
                        query.push(c);
                        anime_state.select_first();

                        let mut filtered_anime = fuzzy_search(
                            &query.to_lowercase(),
                            &anime_list
                                .par_iter()
                                .map(|anime| anime.names[0].to_lowercase().to_string())
                                .collect::<Vec<_>>()
                                .par_iter()
                                .map(String::as_ref)
                                .collect::<Vec<_>>(),
                        )
                        .par_iter()
                        .map(|(_, score)| *score)
                        .zip(anime_list.clone())
                        .collect::<Vec<_>>();

                        filtered_anime.sort_unstable_by(|a, b| {
                            b.0.partial_cmp(&a.0)
                                .expect("Comparison of f32 failed when sorting animes")
                        });

                        *filtered_list = filtered_anime
                            .into_par_iter()
                            .map(|(_, anime)| anime)
                            .collect::<Vec<_>>();
                    }
                }
                _ => {}
            },
            SearchState::Choosing => match code {
                KeyCode::Up | KeyCode::Char('k') => anime_state.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => anime_state.select_next(),
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                    if let Some(i) = anime_state.selected()
                        && let Some(anime) = filtered_list.get(i)
                    {
                        app.menu_state = MenuState::Episodes {
                            state: ListState::default().with_selected(Some(0)),
                            anime_list: mem::take(anime_list),
                            anime: anime.clone(),
                            episodes: match app.config.scraper {
                                ScraperImpl::AnimeAv1Scraper => {
                                    AnimeAv1Scraper::try_get_episodes(&app.client, &anime.names[1])
                                        .expect("Couldn't get episodes")
                                }
                                ScraperImpl::AnimeFlvScraper => {
                                    AnimeFlvScraper::try_get_episodes(&app.client, &anime.names[1])
                                        .expect("Couldn't get episodes")
                                }
                            },
                        };
                    }
                }
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                    *search_state = SearchState::Searching;
                    anime_state.select_first();
                }
                _ => {}
            },
        }
    }
}
