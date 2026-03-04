use std::{fmt, mem};

use iced::{
    Event, Length, Theme,
    alignment::Horizontal,
    event::{self, Status},
    keyboard::{
        Event::KeyPressed,
        Key,
        key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter, Escape},
    },
    widget::{Space, column, container, row, text},
};
use reqwest::blocking::Client;
use strum_macros::EnumIter;

use crate::{
    app,
    config::Config,
    list_query_state::ListQueryState,
    main_menu_page::{self, MainMenuPage},
    page::{AppUpdate, Page},
    presets::{help_text, options_item, square_box},
    scraper::ScraperImpl,
    themes::Themes,
};

#[derive(Debug, Clone)]
pub enum Message {
    UpdateScraper(ScraperImpl),
    UpdateTheme(Themes),
    KeyPressed(Key),
}

#[derive(EnumIter, Default, Clone, Copy)]
pub enum Options {
    #[default]
    Scraper,
    Theme,
}

impl Options {
    pub fn next(&self) -> Self {
        match self {
            Self::Scraper => Self::Theme,
            Self::Theme => Self::Theme,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Scraper => Self::Scraper,
            Self::Theme => Self::Scraper,
        }
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Scraper => "Scraper",
                Self::Theme => "Esquema de colores",
            }
        )
    }
}

pub struct OptionsPage {
    pub old_config: Config,
    pub config: Config,
    pub client: Client,
    pub theme: Theme,
    pub anime_list: ListQueryState,
    pub selection: Options,
}

impl Page for OptionsPage {
    fn view(&self) -> iced::Element<'_, app::Message> {
        square_box(column![
            column![
                options_item::<ScraperImpl, app::Message>(
                    "Scraper: ",
                    matches!(self.selection, Options::Scraper),
                    Some(self.config.scraper.to_string()),
                    |selected| {
                        app::Message::Options(Message::UpdateScraper(
                            selected.parse::<ScraperImpl>().expect("Shouldn't happen"),
                        ))
                    }
                ),
                options_item::<Themes, app::Message>(
                    "Esquema de colores: ",
                    matches!(self.selection, Options::Theme),
                    Some(self.config.theme.to_string()),
                    |selected| {
                        app::Message::Options(Message::UpdateTheme(
                            selected.parse::<Themes>().expect("Shouldn't happen"),
                        ))
                    }
                ),
            ]
            .spacing(6)
            .padding(6),
            Space::with_height(Length::Fill),
            container(row![
                text("Subir:"),
                help_text(" ↑ K "),
                text(" Bajar:"),
                help_text(" ↓ J "),
                text(" Siguiente:"),
                help_text(" → L "),
                text(" Anterior:"),
                help_text(" ← H "),
                text(" Guardar:"),
                help_text(" Enter "),
                text(" Salir sin guardar:"),
                help_text(" Esc Q"),
            ])
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            Space::with_height(Length::Fixed(3.0)),
        ])
        .into()
    }

    fn update(&mut self, message: app::Message) -> AppUpdate {
        if let app::Message::Options(message) = message {
            match message {
                Message::UpdateScraper(scraper) => {
                    self.config.scraper = scraper;
                    AppUpdate::None
                }
                Message::UpdateTheme(theme) => {
                    self.config.theme = theme;
                    self.theme = theme.into();
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
                        Options::Theme => {
                            self.config.theme = self.config.theme.next();
                            self.theme = self.config.theme.into();
                            AppUpdate::None
                        }
                    },
                    Key::Character("h") | Key::Named(ArrowLeft) => match self.selection {
                        Options::Scraper => {
                            self.config.scraper = self.config.scraper.prev();
                            AppUpdate::None
                        }
                        Options::Theme => {
                            self.config.theme = self.config.theme.prev();
                            self.theme = self.config.theme.into();
                            AppUpdate::None
                        }
                    },
                    Key::Named(Enter) => {
                        self.config.save().expect("Couldn't save config");
                        AppUpdate::Page(Box::new(MainMenuPage {
                            config: mem::take(&mut self.config),
                            client: self.client.clone(),
                            theme: mem::take(&mut self.theme),
                            selection: main_menu_page::Selection::Options,
                            anime_list: if self.config.scraper == self.old_config.scraper {
                                mem::take(&mut self.anime_list)
                            } else {
                                ListQueryState::spawn(
                                    self.config.scraper,
                                    mem::take(&mut self.client),
                                )
                            },
                        }))
                    }
                    Key::Character("q") | Key::Named(Escape) => {
                        self.theme = self.old_config.theme.into();
                        AppUpdate::Page(Box::new(MainMenuPage {
                            config: mem::take(&mut self.old_config),
                            client: mem::take(&mut self.client),
                            theme: mem::take(&mut self.theme),
                            selection: main_menu_page::Selection::Options,
                            anime_list: mem::take(&mut self.anime_list),
                        }))
                    }
                    _ => AppUpdate::None,
                },
            }
        } else {
            panic!("options menu event handler called for non options menu event")
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
        self.theme.clone()
    }
}
