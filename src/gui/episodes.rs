use std::{error::Error, io::Stdout, process::Command, time::Duration};

use itertools::Itertools;
use notify_rust::Notification;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Clear, List, ListDirection, ListState},
    Terminal,
};
use reqwest::Client;
use strum::IntoEnumIterator;

use crate::{
    config::Config,
    scraper::{
        anime::Anime, animeav1scraper::AnimeAv1Scraper, animeflvscraper::AnimeFlvScraper, Scraper,
        ScraperImpl,
    },
};

use super::main_menu::MainMenuSelection;

const WHITELIST: [&str; 3] = ["mp4upload", "ok.ru", "my.mail.ru"];

#[allow(clippy::too_many_lines)]
pub async fn episodes(
    config: &Config,
    client: &Client,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    anime: &Anime,
) -> Result<(), Box<dyn Error>> {
    let mut main_menu_state = ListState::default();
    main_menu_state.select_first();

    let mut selected = ListState::default();
    selected.select_first();

    let episode_vec = match config.scraper {
        ScraperImpl::AnimeFlvScraper => {
            AnimeFlvScraper::try_get_episodes(client, &anime.url).await?
        }
        ScraperImpl::AnimeAv1Scraper => {
            AnimeAv1Scraper::try_get_episodes(client, &anime.url).await?
        }
    };

    loop {
        terminal.draw(|frame| {
            frame.render_widget(Clear, frame.area());

            // Divide areas
            let horizontal = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]);

            let [main_menu_area, right_area] = horizontal.areas(frame.area());

            // Render list block
            let list_title =
                Line::from(format!(" Episodios de {} ", anime.name).bold().white()).centered();

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

            let list_block = Block::bordered()
                .title(list_title)
                .title_bottom(list_instructions.centered())
                .border_set(border::THICK)
                .border_style(Style::new().green());

            let episode_list = List::new(episode_vec.iter().map(ToString::to_string))
                .block(list_block)
                .highlight_symbol("> ")
                .highlight_style(Style::new().bold())
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(episode_list, right_area, &mut selected);

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

        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Up | KeyCode::Char('k') => selected.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => selected.select_next(),
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                    if let Some(i) = selected.selected() {
                        let episode = episode_vec[i];
                        let mirrors = match config.scraper {
                            ScraperImpl::AnimeFlvScraper => {
                                AnimeFlvScraper::try_get_mirrors(client, &anime.url, episode)
                                    .await?
                            }
                            ScraperImpl::AnimeAv1Scraper => {
                                AnimeAv1Scraper::try_get_mirrors(client, &anime.url, episode)
                                    .await?
                            }
                        };

                        let viewable = mirrors
                            .iter()
                            .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
                            .collect_vec();

                        let success = viewable.iter().any(|mirror| {
                            Notification::new()
                                .summary("Ani-link")
                                .body(
                                    format!(r#"Abriendo "{mirror}" en mpv, por favor, espera."#)
                                        .as_str(),
                                )
                                .show()
                                .unwrap();

                            let mut command = if cfg!(target_os = "windows") {
                                Command::new("mpv.exe")
                            } else {
                                Command::new("mpv")
                            };

                            command
                                .arg(mirror)
                                .output()
                                .ok()
                                .and_then(|output| output.status.code().filter(|&code| code == 0))
                                .is_some()
                        });

                        if !success {
                            Notification::new()
                                .summary("Ani-link")
                                .body("No se ha podido abrir mpv")
                                .show()
                                .unwrap();
                        }
                        while event::poll(Duration::from_secs(0)).unwrap_or(false) {
                            event::read().unwrap();
                        }
                    }
                }
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    Ok(())
}
