#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{
    io::{BufRead, BufReader},
    mem,
    process::{Command, Stdio},
    sync::{Arc, atomic::Ordering, mpsc::channel},
    thread,
};

use crate::{
    app,
    config::Config,
    image_query_state::ImageQueryState,
    page::{AppUpdate, Page},
    presets::{square_box, transparent_button_cond},
    scraper::anime::Anime,
    search_page::SearchPage,
};
use atomic_float::AtomicF32;
use dirs::{config_dir, state_dir, video_dir};
use iced::{
    Element, Event, Length, Padding, Subscription, Task,
    alignment::Horizontal,
    event::{self, Status},
    keyboard::{
        Event::KeyPressed,
        Key,
        key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter, Escape},
    },
    never,
    time::{self, Duration},
    widget::{
        Column, Id, Scrollable, Space, column, container,
        operation::snap_to,
        progress_bar, rich_text, row,
        scrollable::{self, Direction, Scrollbar},
        span, stack, text,
    },
};
use itertools::Itertools;
use libmpv2::Mpv;
use notify_rust::Notification;
use regex::Regex;
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
    pub selected: usize,
    pub anime_list: Vec<Anime>,
    pub anime: Anime,
    pub episodes: Vec<f64>,
    pub download_progress: Arc<AtomicF32>,
}

impl Page for EpisodesPage {
    fn view(&self) -> iced::Element<'_, crate::app::Message> {
        let selected = self.selected;
        let progress = self.download_progress.load(Ordering::Relaxed);
        stack![
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
                            .id(Id::new(EPISODES_SCROLLABLE_ID))
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
                                span(" → L Enter ").color(self.config.theme().palette().primary),
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
                        .width(Length::Fill),
                    ]
                    .spacing(3)
                    .padding(3)
                )
                .width(Length::Fill),
            ],
            if self.download_progress.load(Ordering::Relaxed).is_nan() {
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
                                    self.episodes[self.selected], self.anime.names[0]
                                )),
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

    fn update(&mut self, message: crate::app::Message) -> AppUpdate {
        if let app::Message::Episodes(message) = message {
            match message {
                Message::Click(index) => {
                    if self.download_progress.load(Ordering::Relaxed).is_nan() {
                        return AppUpdate::None;
                    }

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
                        let image_query = ImageQueryState::spawn(
                            self.client.clone(),
                            self.anime_list
                                .first()
                                .expect("No animes found")
                                .image_url
                                .clone(),
                        );

                        AppUpdate::Page(Box::new(SearchPage {
                            config: mem::take(&mut self.config),
                            client: mem::take(&mut self.client),
                            anime_list: self.anime_list.clone(),
                            query: String::new(),
                            selected: 0,
                            filtered_list: mem::take(&mut self.anime_list),
                            image: image_query,
                            download_progress: Arc::new((
                                AtomicF32::new(0.0),
                                AtomicF32::new(0.0),
                                AtomicF32::new(f32::NAN),
                            )),
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
        let mut subscriptions =
            vec![time::every(Duration::from_millis(100)).map(|_| app::Message::Update)];

        if self.download_progress.load(Ordering::Relaxed).is_nan() {
            subscriptions.push(event::listen_with(move |event, status, _| {
                match (event, status) {
                    (Event::Keyboard(KeyPressed { key, .. }), Status::Ignored) => {
                        Some(app::Message::Episodes(Message::KeyPressed(key)))
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

impl EpisodesPage {
    fn scroll_to_index(&self) -> Task<app::Message> {
        let list_len = self.episodes.len();

        if self.selected >= list_len {
            return Task::none();
        }

        #[allow(clippy::cast_precision_loss)]
        let offset = self.selected as f32 / list_len as f32;

        snap_to(
            Id::new(EPISODES_SCROLLABLE_ID),
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

        let success = viewable.iter().all(|&mirror| {
            Notification::new()
                .summary("Ani-link")
                .body(format!(r#"Abriendo "{mirror}" en mpv, por favor, espera."#).as_str())
                .show()
                .unwrap();

            let (tx, rx) = channel();
            let mirror_clone = mirror.clone();
            let save_on_quit = self.config.save_on_quit;

            thread::spawn(move || {
                let mut sent = false;
                let mirror = mirror_clone;
                let mpv = Mpv::new().unwrap();

                let mut watch_later = state_dir().or_else(config_dir).unwrap();
                watch_later.push("mpv/watch_later");
                let watch_later = watch_later.as_path().to_str().unwrap();

                mpv.set_property("osc", true).unwrap();
                mpv.set_property("watch-later-directory", watch_later)
                    .unwrap();
                mpv.set_property("input-default-bindings", true).unwrap();
                mpv.set_property("force-window", true).unwrap();
                mpv.set_property("keep-open", "no").unwrap();
                mpv.set_property("resume-playback", "yes").unwrap();
                mpv.set_property("idle", "no").unwrap();
                mpv.set_property(
                    "save-position-on-quit",
                    if save_on_quit { "yes" } else { "no" },
                )
                .unwrap();
                mpv.command("loadfile", &[&mirror, "replace"]).unwrap();
                loop {
                    if let Some(Ok(event)) = mpv.wait_event(-1.0) {
                        match event {
                            libmpv2::events::Event::EndFile(reason) => match reason {
                                0 | 3 => break,
                                4 if !sent => {
                                    let _ = tx.send(false).is_ok();
                                    break;
                                }
                                _ if !sent => {
                                    let _ = tx.send(true).is_ok();
                                    sent = true;
                                }
                                _ => {}
                            },
                            libmpv2::events::Event::PlaybackRestart if !sent => {
                                let _ = tx.send(true).is_ok();
                                sent = true;
                            }
                            libmpv2::events::Event::Shutdown if !sent => {
                                let _ = tx.send(false).is_ok();
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                mpv.command("quit", &[]).unwrap();
            });

            rx.recv().unwrap()
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
            .into_iter()
            .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
            .collect_vec();

        let name = self.anime.names[0].clone();
        let slug = self.anime.names[1].clone();

        let progress_counter = self.download_progress.clone();

        thread::spawn(move || {
            progress_counter.store(0.0, Ordering::Relaxed);
            Notification::new()
                .summary("Ani-link")
                .body(
                    format!(r"Descargando episodio {episode} de {name}, por favor, espera.")
                        .as_str(),
                )
                .show()
                .unwrap();

            let success = viewable.into_iter().any(|mirror| {
                let mut command = Command::new(format!(
                    "yt-dlp{}",
                    if cfg!(target_os = "windows") {
                        ".exe"
                    } else {
                        ""
                    }
                ));

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

                let progress_re = Regex::new(r"([0-9.].*)%").unwrap();
                for line in reader.lines() {
                    let line = line.unwrap();
                    let maybe_progress = progress_re.captures_iter(&line).find_map(|c| {
                        let progress = c.get(1)?.as_str();
                        progress.parse::<f32>().ok()
                    });

                    if let Some(progress) = maybe_progress {
                        progress_counter.store(progress, Ordering::Relaxed);
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

            progress_counter.store(f32::NAN, Ordering::Relaxed);
        });
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
