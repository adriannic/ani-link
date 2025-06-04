use fuzzy_matcher::clangd::fuzzy_match;
use itertools::Itertools;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListDirection, ListState, Paragraph};
use ratatui::Terminal;
use reqwest::Client;
use std::error::Error;
use std::io::Stdout;
use strum::IntoEnumIterator;

use crate::config::Config;
use crate::gui::episodes::episodes;
use crate::scraper::anime::Anime;
use crate::scraper::animeav1scraper::AnimeAv1Scraper;
use crate::scraper::animeflvscraper::AnimeFlvScraper;
use crate::scraper::{Scraper, ScraperImpl};

use super::main_menu::MainMenuSelection;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum SearchState {
    Searching,
    Choosing,
}

#[allow(clippy::too_many_lines)]
pub async fn search(
    config: &Config,
    client: &Client,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    const MAX_QUERY: usize = 80;

    let mut main_menu_state = ListState::default();
    main_menu_state.select_first();

    let mut anime_state = ListState::default();
    anime_state.select_first();

    let mut query = String::new();
    query.reserve_exact(MAX_QUERY);

    let mut should_query = true;

    let mut animes: Vec<Anime> = vec![];

    let mut filtered_anime: Vec<Anime> = vec![];

    let mut state = SearchState::Searching;

    loop {
        terminal.draw(|frame| {
            // Divide areas
            let horizontal = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]);

            let [main_menu_area, search_area] = horizontal.areas(frame.area());

            let vertical = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]);

            let [search_area, list_area] = vertical.areas(search_area);

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

            if state == SearchState::Searching {
                search_block = search_block.title_bottom(search_instructions.centered());
            }

            let mut search_text = Paragraph::new(format!(" Nombre: {}", query.as_str()))
                .block(search_block)
                .bold();

            if state == SearchState::Choosing {
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

            if state == SearchState::Choosing {
                list_block = list_block.title_bottom(list_instructions.centered());
            }

            let mut anime_list = List::new(filtered_anime.iter().map(|anime| anime.name.as_str()))
                .block(list_block)
                .highlight_symbol("> ")
                .highlight_style(Style::new().bold())
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            if state == SearchState::Searching {
                anime_list = anime_list.style(Style::new().gray());
            }

            frame.render_stateful_widget(anime_list, list_area, &mut anime_state);

            // Render Main menu option list
            let main_menu_title = Line::from(" Menú principal ".bold().white()).centered();

            let main_menu_block = Block::bordered()
                .title(main_menu_title)
                .border_set(border::THICK)
                .border_style(Style::new().green());

            let main_menu_items = MainMenuSelection::iter().map(|scraper| scraper.to_string());

            let main_menu_list = List::new(main_menu_items)
                .block(main_menu_block)
                .highlight_symbol("> ")
                .highlight_style(Style::new().bold())
                .style(Style::new().gray())
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(main_menu_list, main_menu_area, &mut main_menu_state);
        })?;

        match state {
            SearchState::Searching => {
                if let Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    ..
                }) = event::read()?
                {
                    match code {
                        KeyCode::Esc => break,
                        KeyCode::Enter => state = SearchState::Choosing,
                        KeyCode::Backspace => {
                            if !query.is_empty() {
                                query.pop();
                                if query.is_empty() {
                                    should_query = query.is_empty();
                                    anime_state.select(None);
                                    animes.clear();
                                }

                                filtered_anime = animes
                                    .iter()
                                    .filter(|&anime| fuzzy_match(&anime.name, &query).is_some())
                                    .cloned()
                                    .collect_vec();

                                filtered_anime.sort_by(|a, b| a.name.cmp(&b.name));
                            }
                        }
                        KeyCode::Char(c) => {
                            if query.len() < MAX_QUERY {
                                query.push(c);

                                if should_query {
                                    animes = match config.scraper {
                                        ScraperImpl::AnimeFlvScraper => {
                                            AnimeFlvScraper::try_search(
                                                client,
                                                &query,
                                                config.pages,
                                            )
                                            .await?
                                        }
                                        ScraperImpl::AnimeAv1Scraper => {
                                            AnimeAv1Scraper::try_search(
                                                client,
                                                &query,
                                                config.pages,
                                            )
                                            .await?
                                        }
                                    };
                                    should_query = false;
                                    anime_state.select_first();
                                }

                                filtered_anime = animes
                                    .iter()
                                    .filter(|&anime| fuzzy_match(&anime.name, &query).is_some())
                                    .cloned()
                                    .collect_vec();

                                filtered_anime.sort_by(|a, b| a.name.cmp(&b.name));
                            }
                        }
                        _ => {}
                    }
                }
            }
            SearchState::Choosing => {
                if let Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    ..
                }) = event::read()?
                {
                    match code {
                        KeyCode::Up | KeyCode::Char('k') => anime_state.select_previous(),
                        KeyCode::Down | KeyCode::Char('j') => anime_state.select_next(),
                        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                            if let Some(i) = anime_state.selected() {
                                let anime = filtered_anime.get(i).unwrap();
                                episodes(config, client, terminal, anime).await?;
                            }
                        }
                        KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                            state = SearchState::Searching;
                            anime_state.select_first();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
