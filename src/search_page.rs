use atomic_float::AtomicF32;
use dirs::video_dir;
use iced::{
    Border, Element, Event, Font, Length, Padding, Subscription, Task,
    alignment::{Horizontal, Vertical},
    event::{self, Status},
    keyboard::{
        Event::KeyPressed,
        Key,
        key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter, Escape},
    },
    never,
    widget::{
        Column, Id, Scrollable, Space, column, container, image,
        operation::{focus, focus_next, snap_to},
        progress_bar, rich_text, row,
        scrollable::{self, Direction, Scrollbar},
        span, stack, text, text_input,
    },
};
use itertools::Itertools;
use notify_rust::Notification;
use rayon::prelude::*;
use regex::Regex;
use reqwest::Client;
use rust_fuzzy_search::fuzzy_compare;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{
    env::temp_dir,
    fs::{create_dir_all, write},
    io::{BufRead, BufReader},
    mem,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};
use tokio::runtime::Handle;

use crate::{
    app,
    config::Config,
    episodes_page::{EpisodesPage, WHITELIST},
    image_query_state::ImageQueryState,
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
    pub anime_list: Vec<Anime>,
    pub query: String,
    pub selected: usize,
    pub filtered_list: Vec<Anime>,
    pub image: ImageQueryState,
    pub download_progress: Arc<(AtomicF32, AtomicF32, AtomicF32)>,
}

impl Page for SearchPage {
    #[allow(clippy::too_many_lines)]
    fn view(&self) -> iced::Element<'_, crate::app::Message> {
        let selected = self.selected;
        let anime = &self.filtered_list[self.selected];
        let max = self.download_progress.0.load(Ordering::Relaxed);
        let current = self.download_progress.1.load(Ordering::Relaxed);
        let progress = self.download_progress.2.load(Ordering::Relaxed);
        stack![
            column![
                square_box(
                    column![
                        text_input("Buscar...", &self.query)
                            .id(Id::new(SEARCH_BAR_ID))
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
                                    self.filtered_list.iter().enumerate().map(|(i, anime)| {
                                        let name = anime.names[0].clone();
                                        Element::new(
                                            transparent_button_cond(&name, || selected == i)
                                                .on_press(app::Message::Search(Message::Click(i))),
                                        )
                                    })
                                ))
                                .id(Id::new(SEARCH_SCROLLABLE_ID))
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
                            container(
                                rich_text![
                                    span("Subir:").color(self.config.theme().palette().text),
                                    span(" ↑ K ").color(self.config.theme().palette().primary),
                                    span(" Bajar:").color(self.config.theme().palette().text),
                                    span(" ↓ J ").color(self.config.theme().palette().primary),
                                    span(" Confirmar:").color(self.config.theme().palette().text),
                                    span(" → L Enter ")
                                        .color(self.config.theme().palette().primary),
                                    span(" Buscar:").color(self.config.theme().palette().text),
                                    span(" F / ").color(self.config.theme().palette().primary),
                                    span(" Descargar:").color(self.config.theme().palette().text),
                                    span(" D ").color(self.config.theme().palette().primary),
                                    span(" Syncplay:").color(self.config.theme().palette().text),
                                    span(" S ").color(self.config.theme().palette().primary),
                                    span(" Salir:").color(self.config.theme().palette().text),
                                    span(" ← H Esc Q").color(self.config.theme().palette().primary),
                                ]
                                .on_link_click(never)
                            )
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
                                if let ImageQueryState::Obtained(handle) = &self.image {
                                    column![
                                        column![
                                            image(handle).width(Length::Fill),
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
                                } else {
                                    column![].spacing(6).padding(6)
                                }
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
            ],
            if max == 0.0 {
                row![]
            } else {
                row![
                    Space::new().width(Length::FillPortion(1)),
                    column![
                        Space::new().height(Length::Fill),
                        square_box(
                            column![
                                text(format!(
                                    r#"Descargando episodio {} de "{}"..."#,
                                    current, self.filtered_list[self.selected].names[0]
                                )),
                                text(format!("Episodio: {current}/{max}")),
                                progress_bar(1.0..=max, current),
                                text(format!("Progreso: {progress}%")),
                                progress_bar(0.0..=100.0, progress),
                            ]
                            .padding(10)
                            .spacing(3)
                        )
                        .style(move |theme| {
                            let mut background = theme.palette().background;
                            background.a = 1.0;
                            container::Style {
                                background: Some(iced::Background::Color(background)),
                                ..Default::default()
                            }
                        })
                        .width(Length::Fill)
                        .height(Length::Shrink),
                        Space::new().height(Length::Fill)
                    ]
                    .width(Length::FillPortion(2)),
                    Space::new().width(Length::FillPortion(1)),
                ]
            }
        ]
        .into()
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: crate::app::Message) -> AppUpdate {
        if let app::Message::Search(message) = message {
            match message {
                Message::Update(text) => {
                    self.query = text;
                    self.fuzzy();
                    AppUpdate::None
                }
                Message::Submit => AppUpdate::Task(focus_next()),
                Message::Click(index) => {
                    if self.download_progress.0.load(Ordering::Relaxed) != 0.0 {
                        return AppUpdate::None;
                    }

                    if self.selected != index {
                        self.selected = index;
                        self.image = ImageQueryState::spawn(
                            self.client.clone(),
                            self.filtered_list
                                .get(index)
                                .expect("No animes found")
                                .image_url
                                .clone(),
                        );
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
                        selected: 0,
                        anime_list: mem::take(&mut self.anime_list),
                        anime: anime.clone(),
                        episodes,
                        download_progress: Arc::new(AtomicF32::new(f32::NAN)),
                    }))
                }
                Message::KeyPressed(key) => match key.as_ref() {
                    Key::Character("j") | Key::Named(ArrowDown) => {
                        if self.selected < self.filtered_list.len() - 1 {
                            self.selected += 1;
                            self.image = ImageQueryState::spawn(
                                self.client.clone(),
                                self.filtered_list
                                    .get(self.selected)
                                    .expect("No animes found")
                                    .image_url
                                    .clone(),
                            );
                            return AppUpdate::Task(self.scroll_to_index());
                        }
                        AppUpdate::None
                    }
                    Key::Character("k") | Key::Named(ArrowUp) => {
                        if self.selected > 0 {
                            self.selected -= 1;
                            self.image = ImageQueryState::spawn(
                                self.client.clone(),
                                self.filtered_list
                                    .get(self.selected)
                                    .expect("No animes found")
                                    .image_url
                                    .clone(),
                            );
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
                            selected: 0,
                            anime_list: mem::take(&mut self.anime_list),
                            anime: anime.clone(),
                            episodes,
                            download_progress: Arc::new(AtomicF32::new(f32::NAN)),
                        }))
                    }
                    Key::Character("f" | "/") => {
                        self.selected = 0;
                        AppUpdate::Task(Task::batch([
                            focus(Id::new(SEARCH_BAR_ID)),
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
        } else if matches!(message, app::Message::Update) {
            self.image = mem::take(&mut self.image).get();
            AppUpdate::None
        } else {
            AppUpdate::None
        }
    }

    fn subscription(&self) -> iced::Subscription<crate::app::Message> {
        let mut subscriptions =
            vec![iced::time::every(Duration::from_millis(100)).map(|_| app::Message::Update)];

        if self.download_progress.0.load(Ordering::Relaxed) == 0.0 {
            subscriptions.push(event::listen_with(move |event, status, _| {
                match (event, status) {
                    (Event::Keyboard(KeyPressed { key, .. }), Status::Ignored) => {
                        Some(app::Message::Search(Message::KeyPressed(key)))
                    }
                    _ => None,
                }
            }));
        }

        Subscription::batch(subscriptions)
    }

    fn theme(&self) -> iced::Theme {
        self.config.theme()
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

        snap_to(
            Id::new(SEARCH_SCROLLABLE_ID),
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
        self.image = ImageQueryState::spawn(
            self.client.clone(),
            self.filtered_list
                .get(self.selected)
                .expect("No animes found")
                .image_url
                .clone(),
        );
    }

    #[allow(clippy::too_many_lines)]
    fn download_anime(&self) {
        let anime = self.filtered_list[self.selected].clone();
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

        let client = self.client.clone();
        let scraper = self.config.scraper;
        let mirrors = episodes
            .iter()
            .map(|&episode| {
                Handle::current()
                    .block_on(scraper.try_get_mirrors(&client, &anime.names[1], episode))
                    .expect("Couldn't get mirrors")
            })
            .collect_vec();

        let download_counters = self.download_progress.clone();

        #[allow(clippy::cast_precision_loss)]
        download_counters
            .0
            .store(episodes.len() as f32, Ordering::Relaxed);
        download_counters.1.store(1.0, Ordering::Relaxed);

        thread::spawn(move || {
            let progress_re = Regex::new(r"([0-9.].*)%").unwrap();

            for (episode, mirrors) in episodes.iter().zip(mirrors.iter()) {
                download_counters.2.store(0.0, Ordering::Relaxed);
                let viewable = mirrors
                    .iter()
                    .filter(|&mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
                    .collect_vec();

                let success = viewable.iter().any(|mirror| {
                    let mut command = Command::new(format!(
                        "yt-dlp{}",
                        if cfg!(target_os = "windows") {
                            ".exe"
                        } else {
                            ""
                        }
                    ));

                    let slug = anime.names[1].as_str();

                    #[cfg(target_os = "windows")]
                    command.creation_flags(0x08000000);

                    let mut process = command
                        .arg(mirror)
                        .arg("--no-check-certificates")
                        .arg("--newline")
                        .arg("--output")
                        .arg(format!(
                            "{}/ani-link/{slug}/{slug}-{episode}.%(ext)s",
                            video_dir()
                                .expect("Video path not found")
                                .into_os_string()
                                .into_string()
                                .expect("Video path could not be converted to string"),
                        ))
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                        .expect("couldn't run yt-dlp");

                    let stdout = process.stdout.as_mut().unwrap();
                    let reader = BufReader::new(stdout);

                    for line in reader.lines() {
                        let line = line.unwrap();
                        let maybe_progress = progress_re.captures_iter(&line).find_map(|c| {
                            let progress = c.get(1)?.as_str();
                            progress.parse::<f32>().ok()
                        });

                        if let Some(progress) = maybe_progress {
                            download_counters.2.store(progress, Ordering::Relaxed);
                        }
                    }

                    process.wait().is_ok()
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

                download_counters.1.fetch_add(1.0, Ordering::Relaxed);
            }
            download_counters.0.store(0.0, Ordering::Relaxed);
            download_counters.1.store(0.0, Ordering::Relaxed);
            download_counters.2.store(f32::NAN, Ordering::Relaxed);
        });
    }

    fn stream_anime(&self) {
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
        create_dir_all(path.clone()).expect("couldn't create tmp dir");

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
