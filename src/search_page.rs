use dirs::video_dir;
use iced::{
    Border, Element, Event, Font, Length, Padding, Task, Theme,
    alignment::{Horizontal, Vertical},
    event::{self, Status},
    keyboard::{
        Event::KeyPressed,
        Key,
        key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter, Escape},
    },
    widget::{
        self, Column, Scrollable, column, container, image, rich_text, row,
        scrollable::{self, Direction, Scrollbar},
        span, text, text_input,
    },
};
use itertools::Itertools;
use notify_rust::Notification;
use rayon::prelude::*;
use reqwest::Client;
use rust_fuzzy_search::fuzzy_compare;
use std::{
    env::temp_dir,
    fs::write,
    mem,
    process::{Command, Stdio},
    sync::{Arc, atomic::AtomicUsize},
};
use tokio::runtime::Handle;

use crate::{
    app,
    config::Config,
    episodes_page::{EpisodesPage, WHITELIST},
    list_query_state::ListQueryState,
    main_menu_page::{MainMenuPage, Selection},
    page::{AppUpdate, Page},
    presets::{highlight, square_box, transparent_button_cond},
    scraper::anime::Anime,
};

pub const SEARCH_BAR_ID: &str = "search_bar";
pub const SEARCH_SCROLLABLE_ID: &str = "search_scrollable";

#[derive(Debug, Clone)]
pub enum Message {
    Update(String),
    Click(usize),
    Submit,
    KeyPressed(Key),
}

pub struct SearchPage {
    pub config: Config,
    pub client: Client,
    pub theme: Theme,
    pub anime_list: Vec<Anime>,
    pub query: String,
    pub selected: usize,
    pub filtered_list: Vec<Anime>,
}

impl Page for SearchPage {
    #[allow(clippy::too_many_lines)]
    fn view(&self) -> iced::Element<'_, crate::app::Message> {
        let selected = self.selected;
        let anime = &self.filtered_list[self.selected];
        column![
            square_box(
                column![
                    text_input("Buscar...", &self.query)
                        .id(text_input::Id::new(SEARCH_BAR_ID))
                        .style(move |theme: &iced::Theme, _| text_input::Style {
                            background: iced::Background::Color(theme.palette().background),
                            border: Border::default().width(0),
                            icon: theme.palette().primary,
                            placeholder: highlight(theme.palette().text, 20.0),
                            value: theme.palette().text,
                            selection: theme.palette().primary,
                        })
                        .on_input(|s| app::Message::Search(Message::Update(s)))
                        .on_submit(app::Message::Search(Message::Submit))
                ]
                .spacing(3)
                .padding(3)
            )
            .height(Length::Fixed(39.0)),
            row![
                square_box(
                    column![
                        container(
                            Scrollable::new(Column::with_children(
                                self.filtered_list
                                    .iter()
                                    .enumerate()
                                    .map(|(i, anime)| {
                                        let name = anime.names[0].clone();
                                        Element::new(
                                            transparent_button_cond(&name, || selected == i)
                                                .on_press(app::Message::Search(Message::Click(i))),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            ))
                            .id(scrollable::Id::new(SEARCH_SCROLLABLE_ID))
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
                            span(" Buscar:").color(self.theme.palette().text),
                            span(" F / ").color(self.theme.palette().primary),
                            span(" Descargar:").color(self.theme.palette().text),
                            span(" D ").color(self.theme.palette().primary),
                            span(" Syncplay:").color(self.theme.palette().text),
                            span(" S ").color(self.theme.palette().primary),
                            span(" Salir:").color(self.theme.palette().text),
                            span(" ← H Esc Q").color(self.theme.palette().primary),
                        ])
                        .align_x(Horizontal::Center)
                        .width(Length::Fill)
                        .clip(true),
                    ]
                    .spacing(3)
                    .padding(3)
                )
                .width(Length::FillPortion(2)),
                square_box(container(
                    column![
                        Scrollable::new(
                            column![
                                column![
                                    image(image::Handle::from_bytes(
                                        anime.get_image(self.config.scraper)
                                    ))
                                    .width(Length::Fill),
                                    text(&anime.names[0])
                                        .font(Font {
                                            weight: iced::font::Weight::Bold,
                                            ..Font::DEFAULT
                                        })
                                        .style(|theme: &iced::Theme| text::Style {
                                            color: Some(theme.palette().primary)
                                        })
                                        .width(Length::Fill)
                                        .align_x(Horizontal::Center)
                                        .align_y(Vertical::Bottom)
                                ],
                                text(&anime.synopsis).width(Length::Fill)
                            ]
                            .spacing(6)
                            .padding(6)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .direction(Direction::Vertical(Scrollbar::new()))
                    ]
                    .spacing(6)
                    .padding(6)
                ))
                .width(Length::FillPortion(1))
            ]
        ]
        .into()
    }

    fn update(&mut self, message: crate::app::Message) -> AppUpdate {
        if let app::Message::Search(message) = message {
            match message {
                Message::Update(text) => {
                    self.query = text;
                    self.fuzzy();
                    AppUpdate::None
                }
                Message::Submit => AppUpdate::Task(widget::focus_next()),
                Message::Click(index) => {
                    if self.selected != index {
                        self.selected = index;
                        return AppUpdate::None;
                    }

                    let anime = &self.filtered_list[self.selected];

                    let episodes = Handle::current()
                        .block_on(
                            self.config
                                .scraper
                                .try_get_episodes(&self.client, &anime.names[1]),
                        )
                        .expect("Couldn't get episodes");

                    AppUpdate::Page(Box::new(EpisodesPage {
                        config: mem::take(&mut self.config),
                        client: mem::take(&mut self.client),
                        theme: mem::take(&mut self.theme),
                        selected: 0,
                        anime_list: mem::take(&mut self.anime_list),
                        anime: anime.clone(),
                        episodes,
                    }))
                }
                Message::KeyPressed(key) => match key.as_ref() {
                    Key::Character("j") | Key::Named(ArrowDown) => {
                        if self.selected < self.filtered_list.len() - 1 {
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
                        let anime = &self.filtered_list[self.selected];
                        let episodes = Handle::current()
                            .block_on(
                                self.config
                                    .scraper
                                    .try_get_episodes(&self.client, &anime.names[1]),
                            )
                            .expect("Couldn't get episodes");

                        AppUpdate::Page(Box::new(EpisodesPage {
                            config: mem::take(&mut self.config),
                            client: mem::take(&mut self.client),
                            theme: mem::take(&mut self.theme),
                            selected: 0,
                            anime_list: mem::take(&mut self.anime_list),
                            anime: anime.clone(),
                            episodes,
                        }))
                    }
                    Key::Character("f" | "/") => {
                        self.selected = 0;
                        AppUpdate::Task(Task::batch([
                            text_input::focus(text_input::Id::new(SEARCH_BAR_ID)),
                            self.scroll_to_index(),
                        ]))
                    }
                    Key::Character("d") => {
                        self.download_anime();
                        AppUpdate::None
                    }
                    Key::Character("s") => {
                        self.stream_anime();
                        AppUpdate::None
                    }
                    Key::Character("q" | "h") | Key::Named(Escape | ArrowLeft) => {
                        AppUpdate::Page(Box::new(MainMenuPage {
                            config: mem::take(&mut self.config),
                            client: mem::take(&mut self.client),
                            theme: self.theme.clone(),
                            selection: Selection::Search,
                            anime_list: ListQueryState::Obtained(
                                mem::take(&mut self.anime_list),
                                Arc::new(AtomicUsize::new(self.config.scraper.pages())),
                            ),
                            waiting: false,
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
                Some(app::Message::Search(Message::KeyPressed(key)))
            }
            _ => None,
        })
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}

impl SearchPage {
    fn scroll_to_index(&self) -> Task<app::Message> {
        let list_len = self.filtered_list.len();

        if self.selected >= list_len {
            return Task::none();
        }

        #[allow(clippy::cast_precision_loss)]
        let offset = self.selected as f32 / list_len as f32;

        scrollable::snap_to(
            scrollable::Id::new(SEARCH_SCROLLABLE_ID),
            scrollable::RelativeOffset {
                x: 0.0,
                y: offset.clamp(0.0, 1.0),
            },
        )
    }

    fn fuzzy(&mut self) {
        let mut result = self
            .anime_list
            .as_slice()
            .into_par_iter()
            .filter_map(|anime| {
                anime
                    .names
                    .clone()
                    .into_par_iter()
                    .map(|name| {
                        let name = name.to_lowercase();
                        let score = fuzzy_compare(&self.query, &name);
                        (name, score)
                    })
                    .max_by(|a, b| {
                        a.1.partial_cmp(&b.1)
                            .expect("Error comparing f32 in sort_fuzzy")
                    })
                    .map(|(_, score)| (anime.clone(), score))
            })
            .collect::<Vec<_>>();

        result.par_sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .expect("Error comparing f32 in sort_fuzzy")
        });

        self.filtered_list = result.into_iter().map(|(anime, _)| anime).collect();
    }

    fn download_anime(&mut self) {
        let anime = &self.filtered_list[self.selected];
        let episodes = Handle::current()
            .block_on(
                self.config
                    .scraper
                    .try_get_episodes(&self.client, &anime.names[1]),
            )
            .expect("Couldn't get episodes");

        Notification::new()
            .summary("Ani-link")
            .body(
                format!(
                    r"Descargando todos los episodios de {}, por favor, espera.",
                    anime.names[0]
                )
                .as_str(),
            )
            .show()
            .unwrap();

        for episode in episodes {
            let mirrors = Handle::current()
                .block_on(self.config.scraper.try_get_mirrors(
                    &self.client,
                    &anime.names[1],
                    episode,
                ))
                .expect("Couldn't get mirrors");

            let viewable = mirrors
                .iter()
                .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
                .collect_vec();

            let success = viewable.iter().all(|mirror| {
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
                    .spawn()
                    .is_ok()
            });

            if !success {
                Notification::new()
                    .summary("Ani-link")
                    .body(&format!("No se ha podido descargar el episodio {episode}"))
                    .show()
                    .unwrap();
            }
        }
    }

    fn stream_anime(&mut self) {
        let anime = &self.filtered_list[self.selected];
        let viewable = Handle::current()
            .block_on(
                self.config
                    .scraper
                    .try_get_episodes(&self.client, &anime.names[1]),
            )
            .expect("Couldn't get episodes")
            .iter()
            .flat_map(|&episode| {
                Handle::current()
                    .block_on(self.config.scraper.try_get_mirrors(
                        &self.client,
                        &anime.names[1],
                        episode,
                    ))
                    .expect("Couldn't get mirrors")
                    .iter()
                    .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
                    .map(ToOwned::to_owned)
                    .collect_vec()
            })
            .join("\n");

        let mut path = temp_dir();
        path.push("ani-link");
        path.push("playlist.txt");

        write(&path, viewable).expect("Couldn't create playlist file");

        let mut command = Command::new(format!(
            "syncplay{}",
            if cfg!(target_os = "windows") {
                ".exe"
            } else {
                ""
            }
        ));

        let success = command
            .arg("--load-playlist-from-file")
            .arg(path.to_str().expect("Couldn't convert path to string"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok();

        if !success {
            Notification::new()
                .summary("Ani-link")
                .body("No se ha podido abrir syncplay")
                .show()
                .unwrap();
        }
    }
}
