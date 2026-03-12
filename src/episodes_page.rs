use std::{
    mem,
    process::{Command, Stdio},
};

use crate::{
    app,
    config::Config,
    page::{AppUpdate, Page},
    presets::{square_box, transparent_button_cond},
    scraper::anime::Anime,
    search_page::SearchPage,
};
use dirs::video_dir;
use iced::{
    Element, Event, Length, Padding, Task, Theme,
    alignment::Horizontal,
    event::{self, Status},
    keyboard::{
        Event::KeyPressed,
        Key,
        key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter, Escape},
    },
    widget::{
        Column, Scrollable, column, container, rich_text,
        scrollable::{self, Direction, Scrollbar},
        span,
    },
};
use itertools::Itertools;
use notify_rust::Notification;
use reqwest::Client;
use tokio::runtime::Handle;

const EPISODES_SCROLLABLE_ID: &str = "episodes_scrollable";
pub const WHITELIST: [&str; 3] = ["mp4upload", "ok.ru", "my.mail.ru"];

#[derive(Debug, Clone)]
pub enum Message {
    Click(usize),
    KeyPressed(Key),
}

pub struct EpisodesPage {
    pub config: Config,
    pub client: Client,
    pub theme: Theme,
    pub selected: usize,
    pub anime_list: Vec<Anime>,
    pub anime: Anime,
    pub episodes: Vec<f64>,
}

impl Page for EpisodesPage {
    fn view(&self) -> iced::Element<'_, crate::app::Message> {
        let selected = self.selected;
        column![
            square_box(
                column![
                    container(
                        Scrollable::new(Column::with_children(
                            self.episodes.iter().enumerate().map(|(i, episode)| {
                                Element::new(
                                    transparent_button_cond(&format!("{episode}"), || {
                                        selected == i
                                    })
                                    .on_press(app::Message::Episodes(Message::Click(i))),
                                )
                            })
                        ))
                        .id(scrollable::Id::new(EPISODES_SCROLLABLE_ID))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .direction(Direction::Vertical(Scrollbar::new()))
                    )
                    .padding(Padding {
                        top: 6.0,
                        right: 6.0,
                        bottom: 3.0,
                        left: 6.0
                    }),
                    container(rich_text![
                        span("Subir:").color(self.theme.palette().text),
                        span(" ↑ K ").color(self.theme.palette().primary),
                        span(" Bajar:").color(self.theme.palette().text),
                        span(" ↓ J ").color(self.theme.palette().primary),
                        span(" Confirmar:").color(self.theme.palette().text),
                        span(" → L Enter ").color(self.theme.palette().primary),
                        span(" Descargar:").color(self.theme.palette().text),
                        span(" D ").color(self.theme.palette().primary),
                        span(" Syncplay:").color(self.theme.palette().text),
                        span(" S ").color(self.theme.palette().primary),
                        span(" Salir:").color(self.theme.palette().text),
                        span(" ← H Esc Q").color(self.theme.palette().primary),
                    ])
                    .align_x(Horizontal::Center)
                    .width(Length::Fill),
                ]
                .spacing(3)
                .padding(3)
            )
            .width(Length::Fill),
        ]
        .into()
    }

    fn update(&mut self, message: crate::app::Message) -> AppUpdate {
        if let app::Message::Episodes(message) = message {
            match message {
                Message::Click(index) => {
                    if self.selected != index {
                        self.selected = index;
                        return AppUpdate::None;
                    }

                    self.play_episode();

                    AppUpdate::None
                }
                Message::KeyPressed(key) => match key.as_ref() {
                    Key::Character("j") | Key::Named(ArrowDown) => {
                        if self.selected < self.episodes.len() - 1 {
                            self.selected += 1;
                            return AppUpdate::Task(self.scroll_to_index());
                        }
                        AppUpdate::None
                    }
                    Key::Character("k") | Key::Named(ArrowUp) => {
                        if self.selected > 0 {
                            self.selected -= 1;
                            return AppUpdate::Task(self.scroll_to_index());
                        }
                        AppUpdate::None
                    }
                    Key::Character("l") | Key::Named(ArrowRight | Enter) => {
                        self.play_episode();
                        AppUpdate::None
                    }
                    Key::Character("d") => {
                        self.download_episode();
                        AppUpdate::None
                    }
                    Key::Character("s") => {
                        self.stream_episode();
                        AppUpdate::None
                    }
                    Key::Character("q" | "h") | Key::Named(ArrowLeft | Escape) => {
                        AppUpdate::Page(Box::new(SearchPage {
                            config: mem::take(&mut self.config),
                            client: mem::take(&mut self.client),
                            theme: self.theme.clone(),
                            anime_list: self.anime_list.clone(),
                            query: String::new(),
                            selected: 0,
                            filtered_list: mem::take(&mut self.anime_list),
                        }))
                    }
                    _ => AppUpdate::None,
                },
            }
        } else {
            AppUpdate::None
        }
    }

    fn subscription(&self) -> iced::Subscription<crate::app::Message> {
        event::listen_with(move |event, status, _| match (event, status) {
            (Event::Keyboard(KeyPressed { key, .. }), Status::Ignored) => {
                Some(app::Message::Episodes(Message::KeyPressed(key)))
            }
            _ => None,
        })
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}

impl EpisodesPage {
    fn scroll_to_index(&self) -> Task<app::Message> {
        let list_len = self.episodes.len();

        if self.selected >= list_len {
            return Task::none();
        }

        #[allow(clippy::cast_precision_loss)]
        let offset = self.selected as f32 / list_len as f32;

        scrollable::snap_to(
            scrollable::Id::new(EPISODES_SCROLLABLE_ID),
            scrollable::RelativeOffset {
                x: 0.0,
                y: offset.clamp(0.0, 1.0),
            },
        )
    }

    fn play_episode(&self) {
        let episode = self.episodes[self.selected];

        let mirrors = Handle::current()
            .block_on(self.config.scraper.try_get_mirrors(
                &self.client,
                &self.anime.names[1],
                episode,
            ))
            .expect("Couldn't get mirrors");

        let viewable = mirrors
            .iter()
            .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
            .collect_vec();

        let success = viewable.iter().all(|mirror| {
            Notification::new()
                .summary("Ani-link")
                .body(format!(r#"Abriendo "{mirror}" en mpv, por favor, espera."#).as_str())
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
                .spawn()
                .is_ok()
        });

        if !success {
            Notification::new()
                .summary("Ani-link")
                .body("No se ha podido abrir mpv")
                .show()
                .unwrap();
        }
    }

    fn download_episode(&self) {
        let episode = self.episodes[self.selected];
        let mirrors = Handle::current()
            .block_on(self.config.scraper.try_get_mirrors(
                &self.client,
                &self.anime.names[1],
                episode,
            ))
            .expect("Couldn't get mirrors");

        let viewable = mirrors
            .iter()
            .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
            .collect_vec();

        let success = viewable.iter().all(|mirror| {
            Notification::new()
                .summary("Ani-link")
                .body(
                    format!(
                        r"Descargando episodio {episode} de {}, por favor, espera.",
                        self.anime.names[0]
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

            let slug = self.anime.names[1].as_str();

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
                .spawn()
                .is_ok()
        });

        if success {
            Notification::new()
                .summary("Ani-link")
                .body(&format!("Episodio {episode} descargado correctamente"))
                .show()
                .unwrap();
        } else {
            Notification::new()
                .summary("Ani-link")
                .body(&format!("No se ha podido descargar el episodio {episode}"))
                .show()
                .unwrap();
        }
    }

    fn stream_episode(&self) {
        let episode = self.episodes[self.selected];
        let mirrors = Handle::current()
            .block_on(self.config.scraper.try_get_mirrors(
                &self.client,
                &self.anime.names[1],
                episode,
            ))
            .expect("Couldn't get mirrors");

        let viewable = mirrors
            .iter()
            .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
            .collect_vec();

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
                .spawn()
                .is_ok()
        });

        if !success {
            Notification::new()
                .summary("Ani-link")
                .body("No se ha podido abrir syncplay")
                .show()
                .unwrap();
        }
    }
}
