use std::mem;

use iced::{
    Event, Length,
    alignment::Horizontal,
    event::{self, Status},
    keyboard::{
        Event::KeyPressed,
        Key,
        key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter, Escape},
    },
    widget::{Space, column, container, rich_text, row, span},
};
use reqwest::Client;
use strum_macros::EnumIter;

use crate::{
    app,
    config::Config,
    list_query_state::ListQueryState,
    main_menu_page::{self, MainMenuPage},
    page::{AppUpdate, Page},
    presets::{options_list, options_slider, options_tick, square_box},
    scraper::ScraperImpl,
    themes::Themes,
};

#[derive(Debug, Clone, Copy)]
pub enum Channel {
    Red(f32),
    Green(f32),
    Blue(f32),
    Alpha(f32),
}

impl Default for Channel {
    fn default() -> Self {
        Self::Red(f32::default())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateScraper(ScraperImpl),
    UpdateSaveOnQuit(bool),
    UpdateTheme(Themes),
    Background(Channel),
    Text(Channel),
    Primary(Channel),
    KeyPressed(Key),
}

#[derive(EnumIter, Default, Clone, Copy)]
pub enum Options {
    #[default]
    Scraper,
    SaveOnQuit,
    Theme,
    Background(Channel),
    Text(Channel),
    Primary(Channel),
}

impl Options {
    pub const fn next(self) -> Self {
        match self {
            Self::Scraper => Self::SaveOnQuit,
            Self::SaveOnQuit => Self::Theme,
            Self::Theme => Self::Background(Channel::Red(0.0)),
            Self::Background(Channel::Red(_)) => Self::Background(Channel::Green(0.0)),
            Self::Background(Channel::Green(_)) => Self::Background(Channel::Blue(0.0)),
            Self::Background(Channel::Blue(_)) => Self::Background(Channel::Alpha(0.0)),
            Self::Background(Channel::Alpha(_)) => Self::Text(Channel::Red(0.0)),
            Self::Text(Channel::Red(_)) => Self::Text(Channel::Green(0.0)),
            Self::Text(Channel::Green(_)) => Self::Text(Channel::Blue(0.0)),
            Self::Text(Channel::Blue(_)) => Self::Text(Channel::Alpha(0.0)),
            Self::Text(Channel::Alpha(_)) => Self::Primary(Channel::Red(0.0)),
            Self::Primary(Channel::Red(_)) => Self::Primary(Channel::Green(0.0)),
            Self::Primary(Channel::Green(_)) => Self::Primary(Channel::Blue(0.0)),
            Self::Primary(Channel::Blue(_) | Channel::Alpha(_)) => {
                Self::Primary(Channel::Alpha(0.0))
            }
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Scraper | Self::SaveOnQuit => Self::Scraper,
            Self::Theme => Self::SaveOnQuit,
            Self::Background(Channel::Red(_)) => Self::Theme,
            Self::Background(Channel::Green(_)) => Self::Background(Channel::Red(0.0)),
            Self::Background(Channel::Blue(_)) => Self::Background(Channel::Green(0.0)),
            Self::Background(Channel::Alpha(_)) => Self::Background(Channel::Blue(0.0)),
            Self::Text(Channel::Red(_)) => Self::Background(Channel::Alpha(0.0)),
            Self::Text(Channel::Green(_)) => Self::Text(Channel::Red(0.0)),
            Self::Text(Channel::Blue(_)) => Self::Text(Channel::Green(0.0)),
            Self::Text(Channel::Alpha(_)) => Self::Text(Channel::Blue(0.0)),
            Self::Primary(Channel::Red(_)) => Self::Text(Channel::Alpha(0.0)),
            Self::Primary(Channel::Green(_)) => Self::Primary(Channel::Red(0.0)),
            Self::Primary(Channel::Blue(_)) => Self::Primary(Channel::Green(0.0)),
            Self::Primary(Channel::Alpha(_)) => Self::Primary(Channel::Blue(0.0)),
        }
    }
}

pub struct OptionsPage {
    pub old_config: Config,
    pub config: Config,
    pub client: Client,
    pub anime_list: ListQueryState,
    pub selection: Options,
}

impl Page for OptionsPage {
    #[allow(clippy::too_many_lines)]
    fn view(&self) -> iced::Element<'_, app::Message> {
        square_box(column![
            row![
                column![
                    options_list::<ScraperImpl>(
                        "Scraper: ",
                        matches!(self.selection, Options::Scraper),
                        Some(self.config.scraper.to_string()),
                        |selected| {
                            app::Message::Options(Message::UpdateScraper(
                                selected.parse::<ScraperImpl>().expect("Shouldn't happen"),
                            ))
                        }
                    ),
                    options_tick(
                        "Guardar progreso al salir: ",
                        matches!(self.selection, Options::SaveOnQuit),
                        self.config.save_on_quit,
                        |selected| { app::Message::Options(Message::UpdateSaveOnQuit(selected)) }
                    ),
                    options_list::<Themes>(
                        "Esquema de colores: ",
                        matches!(self.selection, Options::Theme),
                        Some(self.config.theme.to_string()),
                        |selected| {
                            app::Message::Options(Message::UpdateTheme(
                                selected.parse::<Themes>().expect("Shouldn't happen"),
                            ))
                        }
                    ),
                    options_slider(
                        "Color de fondo (rojo): ",
                        matches!(self.selection, Options::Background(Channel::Red(_))),
                        self.config.palette.0.background.r,
                        |v| app::Message::Options(Message::Background(Channel::Red(v)))
                    ),
                    options_slider(
                        "Color de fondo (verde): ",
                        matches!(self.selection, Options::Background(Channel::Green(_))),
                        self.config.palette.0.background.g,
                        |v| app::Message::Options(Message::Background(Channel::Green(v)))
                    ),
                    options_slider(
                        "Color de fondo (azul): ",
                        matches!(self.selection, Options::Background(Channel::Blue(_))),
                        self.config.palette.0.background.b,
                        |v| app::Message::Options(Message::Background(Channel::Blue(v)))
                    ),
                    options_slider(
                        "Color de fondo (opacidad): ",
                        matches!(self.selection, Options::Background(Channel::Alpha(_))),
                        self.config.palette.0.background.a,
                        |v| app::Message::Options(Message::Background(Channel::Alpha(v)))
                    ),
                    options_slider(
                        "Color de texto (rojo): ",
                        matches!(self.selection, Options::Text(Channel::Red(_))),
                        self.config.palette.0.text.r,
                        |v| app::Message::Options(Message::Text(Channel::Red(v)))
                    ),
                    options_slider(
                        "Color de texto (verde): ",
                        matches!(self.selection, Options::Text(Channel::Green(_))),
                        self.config.palette.0.text.g,
                        |v| app::Message::Options(Message::Text(Channel::Green(v)))
                    ),
                    options_slider(
                        "Color de texto (azul): ",
                        matches!(self.selection, Options::Text(Channel::Blue(_))),
                        self.config.palette.0.text.b,
                        |v| app::Message::Options(Message::Text(Channel::Blue(v)))
                    ),
                    options_slider(
                        "Color de texto (opacidad): ",
                        matches!(self.selection, Options::Text(Channel::Alpha(_))),
                        self.config.palette.0.text.a,
                        |v| app::Message::Options(Message::Text(Channel::Alpha(v)))
                    ),
                    options_slider(
                        "Color de acento (rojo): ",
                        matches!(self.selection, Options::Primary(Channel::Red(_))),
                        self.config.palette.0.primary.r,
                        |v| app::Message::Options(Message::Primary(Channel::Red(v)))
                    ),
                    options_slider(
                        "Color de acento (verde): ",
                        matches!(self.selection, Options::Primary(Channel::Green(_))),
                        self.config.palette.0.primary.g,
                        |v| app::Message::Options(Message::Primary(Channel::Green(v)))
                    ),
                    options_slider(
                        "Color de acento (azul): ",
                        matches!(self.selection, Options::Primary(Channel::Blue(_))),
                        self.config.palette.0.primary.b,
                        |v| app::Message::Options(Message::Primary(Channel::Blue(v)))
                    ),
                    options_slider(
                        "Color de acento (opacidad): ",
                        matches!(self.selection, Options::Primary(Channel::Alpha(_))),
                        self.config.palette.0.primary.a,
                        |v| app::Message::Options(Message::Primary(Channel::Alpha(v)))
                    ),
                ]
                .spacing(6)
                .padding(6),
                Space::with_width(Length::Fixed(18.0))
            ],
            Space::with_height(Length::Fill),
            container(rich_text![
                span("Subir:").color(self.config.theme().palette().text),
                span(" ↑ K ").color(self.config.theme().palette().primary),
                span(" Bajar:").color(self.config.theme().palette().text),
                span(" ↓ J ").color(self.config.theme().palette().primary),
                span(" Siguiente:").color(self.config.theme().palette().text),
                span(" → L ").color(self.config.theme().palette().primary),
                span(" Anterior:").color(self.config.theme().palette().text),
                span(" ← H ").color(self.config.theme().palette().primary),
                span(" Guardar:").color(self.config.theme().palette().text),
                span(" Enter ").color(self.config.theme().palette().primary),
                span(" Salir sin guardar:").color(self.config.theme().palette().text),
                span(" Esc Q").color(self.config.theme().palette().primary),
            ])
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            Space::with_height(Length::Fixed(3.0)),
        ])
        .into()
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: app::Message) -> AppUpdate {
        if let app::Message::Options(message) = message {
            match message {
                Message::UpdateScraper(scraper) => {
                    self.config.scraper = scraper;
                    AppUpdate::None
                }
                Message::UpdateSaveOnQuit(selected) => {
                    self.config.save_on_quit = selected;
                    AppUpdate::None
                }
                Message::UpdateTheme(theme) => {
                    self.config.theme = theme;
                    self.config.palette = self.theme().palette().into();
                    AppUpdate::None
                }
                Message::Background(channel) => {
                    match channel {
                        Channel::Red(v) => {
                            self.config.palette.0.background.r = v;
                        }
                        Channel::Green(v) => {
                            self.config.palette.0.background.g = v;
                        }
                        Channel::Blue(v) => {
                            self.config.palette.0.background.b = v;
                        }
                        Channel::Alpha(v) => {
                            self.config.palette.0.background.a = v;
                        }
                    }
                    self.config.theme = Themes::Custom;
                    AppUpdate::None
                }
                Message::Text(channel) => {
                    match channel {
                        Channel::Red(v) => {
                            self.config.palette.0.text.r = v;
                        }
                        Channel::Green(v) => {
                            self.config.palette.0.text.g = v;
                        }
                        Channel::Blue(v) => {
                            self.config.palette.0.text.b = v;
                        }
                        Channel::Alpha(v) => {
                            self.config.palette.0.text.a = v;
                        }
                    }
                    self.config.theme = Themes::Custom;
                    AppUpdate::None
                }
                Message::Primary(channel) => {
                    match channel {
                        Channel::Red(v) => {
                            self.config.palette.0.primary.r = v;
                        }
                        Channel::Green(v) => {
                            self.config.palette.0.primary.g = v;
                        }
                        Channel::Blue(v) => {
                            self.config.palette.0.primary.b = v;
                        }
                        Channel::Alpha(v) => {
                            self.config.palette.0.primary.a = v;
                        }
                    }
                    self.config.theme = Themes::Custom;
                    AppUpdate::None
                }
                Message::KeyPressed(key) => match key.as_ref() {
                    Key::Character("j") | Key::Named(ArrowDown) => {
                        self.selection = self.selection.next();
                        AppUpdate::None
                    }
                    Key::Character("k") | Key::Named(ArrowUp) => {
                        self.selection = self.selection.prev();
                        AppUpdate::None
                    }
                    Key::Character("l") | Key::Named(ArrowRight) => match self.selection {
                        Options::Scraper => {
                            self.config.scraper = self.config.scraper.next();
                            AppUpdate::None
                        }
                        Options::SaveOnQuit => {
                            self.config.save_on_quit = !self.config.save_on_quit;
                            AppUpdate::None
                        }
                        Options::Theme => {
                            self.config.theme = self.config.theme.next();
                            self.config.palette = self.theme().palette().into();
                            AppUpdate::None
                        }
                        Options::Background(channel) => {
                            match channel {
                                Channel::Red(_) => {
                                    let v = self.config.palette.0.background.r;
                                    self.config.palette.0.background.r =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Green(_) => {
                                    let v = self.config.palette.0.background.g;
                                    self.config.palette.0.background.g =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Blue(_) => {
                                    let v = self.config.palette.0.background.b;
                                    self.config.palette.0.background.b =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Alpha(_) => {
                                    let v = self.config.palette.0.background.a;
                                    self.config.palette.0.background.a =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                            }
                            self.config.theme = Themes::Custom;
                            AppUpdate::None
                        }
                        Options::Text(channel) => {
                            match channel {
                                Channel::Red(_) => {
                                    let v = self.config.palette.0.text.r;
                                    self.config.palette.0.text.r =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Green(_) => {
                                    let v = self.config.palette.0.text.g;
                                    self.config.palette.0.text.g =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Blue(_) => {
                                    let v = self.config.palette.0.text.b;
                                    self.config.palette.0.text.b =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Alpha(_) => {
                                    let v = self.config.palette.0.text.a;
                                    self.config.palette.0.text.a =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                            }
                            self.config.theme = Themes::Custom;
                            AppUpdate::None
                        }
                        Options::Primary(channel) => {
                            match channel {
                                Channel::Red(_) => {
                                    let v = self.config.palette.0.primary.r;
                                    self.config.palette.0.primary.r =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Green(_) => {
                                    let v = self.config.palette.0.primary.g;
                                    self.config.palette.0.primary.g =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Blue(_) => {
                                    let v = self.config.palette.0.primary.b;
                                    self.config.palette.0.primary.b =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Alpha(_) => {
                                    let v = self.config.palette.0.primary.a;
                                    self.config.palette.0.primary.a =
                                        (v + 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                            }
                            self.config.theme = Themes::Custom;
                            AppUpdate::None
                        }
                    },
                    Key::Character("h") | Key::Named(ArrowLeft) => match self.selection {
                        Options::Scraper => {
                            self.config.scraper = self.config.scraper.prev();
                            AppUpdate::None
                        }
                        Options::SaveOnQuit => {
                            self.config.save_on_quit = !self.config.save_on_quit;
                            AppUpdate::None
                        }
                        Options::Theme => {
                            self.config.theme = self.config.theme.prev();
                            self.config.palette = self.theme().palette().into();
                            AppUpdate::None
                        }
                        Options::Background(channel) => {
                            match channel {
                                Channel::Red(_) => {
                                    let v = self.config.palette.0.background.r;
                                    self.config.palette.0.background.r =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Green(_) => {
                                    let v = self.config.palette.0.background.g;
                                    self.config.palette.0.background.g =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Blue(_) => {
                                    let v = self.config.palette.0.background.b;
                                    self.config.palette.0.background.b =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Alpha(_) => {
                                    let v = self.config.palette.0.background.a;
                                    self.config.palette.0.background.a =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                            }
                            self.config.theme = Themes::Custom;
                            AppUpdate::None
                        }
                        Options::Text(channel) => {
                            match channel {
                                Channel::Red(_) => {
                                    let v = self.config.palette.0.text.r;
                                    self.config.palette.0.text.r =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Green(_) => {
                                    let v = self.config.palette.0.text.g;
                                    self.config.palette.0.text.g =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Blue(_) => {
                                    let v = self.config.palette.0.text.b;
                                    self.config.palette.0.text.b =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Alpha(_) => {
                                    let v = self.config.palette.0.text.a;
                                    self.config.palette.0.text.a =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                            }
                            self.config.theme = Themes::Custom;
                            AppUpdate::None
                        }
                        Options::Primary(channel) => {
                            match channel {
                                Channel::Red(_) => {
                                    let v = self.config.palette.0.primary.r;
                                    self.config.palette.0.primary.r =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Green(_) => {
                                    let v = self.config.palette.0.primary.g;
                                    self.config.palette.0.primary.g =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Blue(_) => {
                                    let v = self.config.palette.0.primary.b;
                                    self.config.palette.0.primary.b =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                                Channel::Alpha(_) => {
                                    let v = self.config.palette.0.primary.a;
                                    self.config.palette.0.primary.a =
                                        (v - 1.0 / 255.0).clamp(0.0, 1.0);
                                }
                            }
                            self.config.theme = Themes::Custom;
                            AppUpdate::None
                        }
                    },
                    Key::Named(Enter) => {
                        let anime_list = if self.config.scraper == self.old_config.scraper {
                            mem::take(&mut self.anime_list)
                        } else {
                            ListQueryState::spawn(self.config.scraper, mem::take(&mut self.client))
                        };

                        self.config.save().expect("Couldn't save config");

                        AppUpdate::Page(Box::new(MainMenuPage {
                            config: mem::take(&mut self.config),
                            client: self.client.clone(),
                            selection: main_menu_page::Selection::Options,
                            anime_list,
                            waiting: false,
                        }))
                    }
                    Key::Character("q") | Key::Named(Escape) => {
                        self.config.theme = self.old_config.theme;
                        AppUpdate::Page(Box::new(MainMenuPage {
                            config: mem::take(&mut self.old_config),
                            client: mem::take(&mut self.client),
                            selection: main_menu_page::Selection::Options,
                            anime_list: mem::take(&mut self.anime_list),
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

    fn subscription(&self) -> iced::Subscription<app::Message> {
        event::listen_with(move |event, status, _| match (event, status) {
            (Event::Keyboard(KeyPressed { key, .. }), Status::Ignored) => {
                Some(app::Message::Options(Message::KeyPressed(key)))
            }
            _ => None,
        })
    }

    fn theme(&self) -> iced::Theme {
        self.config.theme()
    }
}
