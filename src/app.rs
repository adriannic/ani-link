#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, atomic::Ordering, mpsc::channel},
};

use dirs::video_dir;
use iced::{
    Font, Length, Settings, Task,
    widget::{Space, column, container, progress_bar, row, stack, text},
};
use itertools::Itertools;
use notify_rust::Notification;
use regex::Regex;
use reqwest::Client;

use crate::{
    config::Config,
    download::{Download, DownloadToken},
    episodes_page::{self, WHITELIST},
    list_query_state::ListQueryState,
    main_menu_page::{self, MainMenuPage},
    options_page,
    page::{AppUpdate, Page},
    presets::square_box,
    search_page,
};

#[derive(Debug, Clone)]
pub enum Message {
    Update,
    Download(Vec<DownloadToken>),
    MainMenu(main_menu_page::Message),
    Options(options_page::Message),
    Search(search_page::Message),
    Episodes(episodes_page::Message),
}

pub struct App {
    pub page: Box<dyn Page>,
    pub download: Arc<Download>,
}

impl Default for App {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        let config: Config = Config::init().expect("Couldn't initialize config");
        let config2 = config.clone();

        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            )
            .cookie_store(true)
            .build()
            .unwrap_or_else(|err| {
                eprintln!("{err}");
                Client::default()
            });
        let client2 = client.clone();

        let scraper = config.scraper;

        let anime_list = ListQueryState::spawn(scraper, client.clone());

        let (tx, rx) = channel();

        let download = Arc::new(Download::new(tx));
        let download2 = download.clone();

        let progress_re = Regex::new(r"([0-9.].*)%").unwrap();

        tokio::spawn(async move {
            while let Ok(download_token) = rx.recv() {
                download.progress().store(0.0, Ordering::Relaxed);
                *download.current() = Some(download_token.clone());

                let DownloadToken {
                    name,
                    slug,
                    episode,
                } = download_token;

                let mirrors = config
                    .scraper
                    .try_get_mirrors(&client, &slug, episode)
                    .await
                    .expect("Couldn't get mirrors")
                    .into_iter()
                    .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
                    .collect_vec();

                let _ = Notification::new()
                    .summary("Ani-link")
                    .body(format!(r"Descargando episodio {episode} de {name}...").as_str())
                    .show()
                    .is_ok();

                let success = mirrors.into_iter().any(|mirror| {
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

                    for line in reader.lines() {
                        let line = line.unwrap();
                        let maybe_progress = progress_re.captures_iter(&line).find_map(|c| {
                            let progress = c.get(1)?.as_str();
                            progress.parse::<f32>().ok()
                        });

                        if let Some(progress) = maybe_progress {
                            download.progress().store(progress, Ordering::Relaxed);
                        }
                    }

                    process.wait().is_ok()
                });

                if success {
                    let _ = Notification::new()
                        .summary("Ani-link")
                        .body(&format!(
                            "Episodio {episode} de {name} descargado correctamente"
                        ))
                        .show()
                        .is_ok();
                } else {
                    let _ = Notification::new()
                        .summary("Ani-link")
                        .body(&format!(
                            "No se ha podido descargar el episodio {episode} de {name}"
                        ))
                        .show()
                        .is_ok();
                }

                download.progress().store(f32::NAN, Ordering::Relaxed);
                *download.current() = None;
            }
        });

        Self {
            page: Box::new(MainMenuPage {
                config: config2,
                client: client2,
                selection: main_menu_page::Selection::Search,
                anime_list,
                waiting: false,
            }),
            download: download2,
        }
    }
}

impl App {
    #[allow(clippy::missing_errors_doc)]
    pub fn run() -> iced::Result {
        iced::application(Self::default, Self::update, Self::view)
            .theme(|app: &Self| app.page.theme())
            .subscription(Self::subscription)
            .transparent(true)
            .antialiasing(true)
            .settings(Settings::default())
            .font(include_bytes!("../assets/font.ttf"))
            .default_font(Font {
                weight: iced::font::Weight::Normal,
                ..Font::with_name("FiraCode Nerd Font Mono")
            })
            .run()
    }

    pub(crate) fn view(&self) -> iced::Element<'_, Message> {
        let progress = self.download.progress().load(Ordering::Relaxed);
        let maybe_current = self.download.current().clone();

        stack![
            self.page.view(),
            if let Some(current) = maybe_current {
                row![
                    Space::new().width(Length::FillPortion(2)),
                    column![
                        Space::new().height(Length::Fill),
                        square_box(
                            column![
                                text(format!("{} episodio {}", current.name, current.episode)),
                                row![
                                    progress_bar(0.0..=100.0, progress),
                                    Space::new().width(Length::Fixed(3.0)),
                                    text(format!("{progress}%")).width(Length::Fixed(50.0)),
                                ],
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
                    ]
                    .width(Length::FillPortion(1)),
                ]
            } else {
                row![]
            }
        ]
        .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::Download(tokens) = message {
            for token in tokens {
                self.download
                    .tx()
                    .send(token)
                    .expect("Couldn't send download token");
            }

            Task::none()
        } else {
            let update = self.page.update(message);
            match update {
                AppUpdate::Page(page) => {
                    self.page = page;
                    Task::none()
                }
                AppUpdate::Task(task) => task,
                AppUpdate::Both((page, task)) => {
                    self.page = page;
                    task
                }
                AppUpdate::None => Task::none(),
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        self.page.subscription()
    }
}
