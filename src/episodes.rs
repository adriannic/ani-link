use std::{mem, process::Command, time::Duration};

use itertools::Itertools;
use notify_rust::Notification;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListDirection, ListState},
};

use crate::{
    app::App,
    menu_state::MenuState,
    scraper::{
        Scraper, ScraperImpl, anime::Anime, animeav1scraper::AnimeAv1Scraper,
        animeflvscraper::AnimeFlvScraper,
    },
    search::SearchState,
};

const WHITELIST: [&str; 3] = ["mp4upload", "ok.ru", "my.mail.ru"];

pub fn draw_episodes(
    frame: &mut Frame,
    content_area: Rect,
    anime: &Anime,
    state: &mut ListState,
    episodes: &[f64],
) {
    // Render list block
    let list_title =
        Line::from(format!(" Episodios de {} ", anime.names[0]).bold().white()).centered();

    let list_instructions = Line::from(vec![
        " Subir:".white(),
        " ↑ K ".blue().bold(),
        " Bajar:".white(),
        " ↓ J ".blue().bold(),
        " Confirmar:".white(),
        " → L Enter ".blue().bold(),
        " Syncplay:".white(),
        " S ".blue().bold(),
        " Atrás:".white(),
        " ← H Esc ".blue().bold(),
    ]);

    let list_block = Block::bordered()
        .title(list_title)
        .title_bottom(list_instructions.centered())
        .border_set(border::THICK)
        .border_style(Style::new().green());

    let episode_list = List::new(episodes.iter().map(ToString::to_string))
        .block(list_block)
        .highlight_symbol("> ")
        .highlight_style(Style::new().bold())
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    frame.render_stateful_widget(episode_list, content_area, state);
}

pub fn handle_events_episodes(app: &mut App) {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = event::read().expect("Couldn't read event")
    {
        let MenuState::Episodes {
            state,
            anime_list,
            anime,
            episodes,
        } = &mut app.menu_state
        else {
            panic!("Invalid app state in episodes")
        };

        match code {
            KeyCode::Up | KeyCode::Char('k') => state.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => state.select_next(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('s') | KeyCode::Enter => {
                let use_syncplay = matches!(code, KeyCode::Char('s'));
                if let Some(i) = state.selected() {
                    let episode = episodes[i];
                    let mirrors = match app.config.scraper {
                        ScraperImpl::AnimeAv1Scraper => {
                            AnimeAv1Scraper::try_get_mirrors(&app.client, &anime.names[1], episode)
                                .expect("Couldn't get mirrors")
                        }
                        ScraperImpl::AnimeFlvScraper => {
                            AnimeFlvScraper::try_get_mirrors(&app.client, &anime.names[1], episode)
                                .expect("Couldn't get mirrors")
                        }
                    };

                    let viewable = mirrors
                        .iter()
                        .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
                        .collect_vec();

                    let success = !viewable.iter().all(|mirror| {
                        Notification::new()
                            .summary("Ani-link")
                            .body(
                                format!(
                                    r#"Abriendo "{mirror}" en {}, por favor, espera."#,
                                    if use_syncplay { "syncplay" } else { "mpv" }
                                )
                                .as_str(),
                            )
                            .show()
                            .unwrap();

                        let mut command = if use_syncplay {
                            Command::new(format!(
                                "syncplay{}",
                                if cfg!(target_os = "windows") {
                                    ".exe"
                                } else {
                                    ""
                                }
                            ))
                        } else {
                            Command::new(format!(
                                "mpv{}",
                                if cfg!(target_os = "windows") {
                                    ".exe"
                                } else {
                                    ""
                                }
                            ))
                        };

                        command
                            .arg(mirror)
                            .output()
                            .ok()
                            .and_then(|output| output.status.code().filter(|&code| code == 0))
                            .is_none()
                    });

                    if !success {
                        Notification::new()
                            .summary("Ani-link")
                            .body(&format!(
                                "No se ha podido abrir {}",
                                if use_syncplay { "syncplay" } else { "mpv" }
                            ))
                            .show()
                            .unwrap();
                    }
                    while event::poll(Duration::from_secs(0)).unwrap_or(false) {
                        event::read().unwrap();
                    }
                }
            }

            KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                let anime_list = mem::take(anime_list);

                app.menu_state = MenuState::Search {
                    anime_list: anime_list.clone(),
                    search_state: SearchState::Searching,
                    query: String::new(),
                    anime_state: ListState::default().with_selected(Some(0)),
                    filtered_list: anime_list,
                }
            }
            _ => {}
        }
    }
}
