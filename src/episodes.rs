use std::{
    mem,
    process::{Command, Stdio},
    time::Duration,
};

use dirs::video_dir;
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
    menu_state::{MenuState, PopupState},
    popup::{Popup, get_popup_area},
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
    popup_state: PopupState,
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
        " Descargar:".white(),
        " D ".blue().bold(),
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

    let popup = match popup_state {
        PopupState::Syncplay(episode) => Some(Popup::new(
            " Syncplay ".into(),
            format!("\nAbriendo el episodio {episode} en syncplay..."),
        )),
        PopupState::Download(episode) => Some(Popup::new(
            " Descarga ".into(),
            format!("\nDescargando el episodio {episode}..."),
        )),
        PopupState::Mpv(episode) => Some(Popup::new(
            " Abriendo ".into(),
            format!("\nAbriendo el episodio {episode} en mpv..."),
        )),
        PopupState::None => None,
    };

    if let Some(popup) = popup {
        frame.render_widget(popup, get_popup_area(frame.area(), 40, 5));
    }
}

pub fn handle_events_episodes(app: &mut App) {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = event::read().expect("Couldn't read event")
    {
        let MenuState::Episodes {
            ref mut state,
            ref mut anime_list,
            ref mut anime,
            ref mut episodes,
            ..
        } = app.menu_state
        else {
            panic!("Invalid app state in episodes")
        };

        match code {
            KeyCode::Up | KeyCode::Char('k') => state.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => state.select_next(),
            KeyCode::Char('s') => {
                let _ = anime_list;
                if let Some(i) = state.selected() {
                    let _ = state;
                    let episode = episodes[i];
                    let _ = episodes;
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
                    let _ = anime;

                    let viewable = mirrors
                        .iter()
                        .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
                        .collect_vec();

                    let MenuState::Episodes { popup_state, .. } = &mut app.menu_state else {
                        panic!("Invalid app state")
                    };
                    *popup_state = PopupState::Syncplay(episode);
                    app.draw().unwrap();

                    let success = viewable.iter().any(|mirror| {
                        let mut command = Command::new(format!(
                            "syncplay{}",
                            if cfg!(target_os = "windows") {
                                ".exe"
                            } else {
                                ""
                            }
                        ));

                        command
                            .arg(mirror)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .is_ok()
                    });

                    if !success {
                        Notification::new()
                            .summary("Ani-link")
                            .body("No se ha podido abrir syncplay")
                            .show()
                            .unwrap();
                    }
                    while event::poll(Duration::from_secs(0)).unwrap_or(false) {
                        event::read().unwrap();
                    }

                    let MenuState::Episodes { popup_state, .. } = &mut app.menu_state else {
                        panic!("Invalid app state")
                    };
                    *popup_state = PopupState::None;
                    app.draw().unwrap();
                }
            }
            KeyCode::Char('d') => {
                let _ = anime_list;
                if let Some(i) = state.selected() {
                    let _ = state;
                    let episode = episodes[i];
                    let _ = episodes;
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
                    let anime = anime.clone();

                    let viewable = mirrors
                        .iter()
                        .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
                        .collect_vec();

                    let MenuState::Episodes { popup_state, .. } = &mut app.menu_state else {
                        panic!("Invalid app state")
                    };
                    *popup_state = PopupState::Download(episode);
                    app.draw().unwrap();

                    let success = viewable.iter().all(|mirror| {
                        Notification::new()
                            .summary("Ani-link")
                            .body(
                                format!(
                                    r#"Descargando episodio {episode} de {}, por favor, espera."#,
                                    anime.names[0]
                                )
                                .as_str(),
                            )
                            .show()
                            .unwrap();

                        let mut command = Command::new(format!(
                            "yt-dlp{}",
                            if cfg!(target_os = "windows") {
                                ".exe"
                            } else {
                                ""
                            }
                        ));

                        let slug = anime.names[1].as_str();

                        command
                            .arg(mirror)
                            .arg("--no-check-certificates")
                            .arg("--output")
                            .arg(format!(
                                "{}/ani-link/{slug}/{slug}-{episode}.%(ext)s",
                                video_dir()
                                    .expect("Video path not found")
                                    .into_os_string()
                                    .into_string()
                                    .expect("Video path could not be converted to string"),
                            ))
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .is_ok()
                    });

                    let MenuState::Episodes { popup_state, .. } = &mut app.menu_state else {
                        panic!("Invalid app state")
                    };
                    *popup_state = PopupState::None;
                    app.draw().unwrap();

                    if !success {
                        Notification::new()
                            .summary("Ani-link")
                            .body(&format!("No se ha podido descargar el episodio {episode}"))
                            .show()
                            .unwrap();
                    }
                    while event::poll(Duration::from_secs(0)).unwrap_or(false) {
                        event::read().unwrap();
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
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

                    let success = viewable.iter().all(|mirror| {
                        let MenuState::Episodes { popup_state, .. } = &mut app.menu_state else {
                            panic!("Invalid app state")
                        };
                        *popup_state = PopupState::Mpv(episode);
                        app.draw().unwrap();

                        Notification::new()
                            .summary("Ani-link")
                            .body(
                                format!(r#"Abriendo "{mirror}" en mpv, por favor, espera."#)
                                    .as_str(),
                            )
                            .show()
                            .unwrap();

                        let mut command = Command::new(format!(
                            "mpv{}",
                            if cfg!(target_os = "windows") {
                                ".exe"
                            } else {
                                ""
                            }
                        ));

                        command
                            .arg(mirror)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .is_ok()
                    });

                    let MenuState::Episodes { popup_state, .. } = &mut app.menu_state else {
                        panic!("Invalid app state")
                    };
                    *popup_state = PopupState::None;
                    app.draw().unwrap();

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
