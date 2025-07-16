use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use fuzzy_matcher::clangd::fuzzy_match;
use itertools::Itertools;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListDirection, ListState, Paragraph};
use ratatui::Terminal;
use reqwest::Client;
use std::error::Error;
use std::io::Stdout;
use strum::IntoEnumIterator;

use crate::config::Config;
use crate::gui::episodes::episodes;
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
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
        )
        .cookie_store(true)
        .build()?;

    const MAX_QUERY: usize = 80;

    let mut main_menu_state = ListState::default();
    main_menu_state.select_first();

    let mut anime_state = ListState::default();
    anime_state.select_first();

    let mut query = String::new();
    query.reserve_exact(MAX_QUERY);

    let animes = match config.scraper {
        ScraperImpl::AnimeAv1Scraper => AnimeAv1Scraper::try_search(&client).await?,
        ScraperImpl::AnimeFlvScraper => AnimeFlvScraper::try_search(&client).await?,
    };

    let mut events = EventStream::default();

    let mut state = SearchState::Searching;

    loop {
        let filtered_anime = animes
            .iter()
            .filter(|&anime| fuzzy_match(&anime.name, &query).is_some())
            .cloned()
            .sorted()
            .collect_vec();

        terminal.draw(|frame| {
            frame.render_widget(Clear, frame.area());

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
                tokio::select! {
                    event = events.next().fuse() => {
                        if let Some(Ok(Event::Key(KeyEvent {
                            code,
                            kind: KeyEventKind::Press,
                            ..
                        }))) = event
                        {
                            match code {
                                KeyCode::Esc => break,
                                KeyCode::Enter => state = SearchState::Choosing,
                                KeyCode::Backspace => {
                                    if !query.is_empty() {
                                        anime_state.select_first();
                                        query.pop();
                                        if query.is_empty() {
                                            anime_state.select(None);
                                        }
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if query.len() < MAX_QUERY {
                                        query.push(c);
                                        anime_state.select_first();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {}
                }
            }
            SearchState::Choosing => {
                tokio::select! {
                    event = events.next().fuse() => {
                        if let Some(Ok(Event::Key(KeyEvent {
                            code,
                            kind: KeyEventKind::Press,
                            ..
                        }))) = event
                        {
                            match code {
                                KeyCode::Up | KeyCode::Char('k') => anime_state.select_previous(),
                                KeyCode::Down | KeyCode::Char('j') => anime_state.select_next(),
                                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                                    if let Some(i) = anime_state.selected() {
                                        if let Some(anime) = filtered_anime.get(i) {
                                            episodes(config, &client, terminal, anime).await?;
                                        }
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
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {}
                }
            }
        }
    }

    Ok(())
}
