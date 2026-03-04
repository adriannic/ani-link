use std::fmt;
use std::mem;
use std::process::exit;

use iced::Event;
use iced::Font;
use iced::Length;
use iced::Theme;
use iced::alignment::Horizontal;
use iced::event;
use iced::event::Status;
use iced::keyboard::Event::KeyPressed;
use iced::keyboard::Key;
use iced::keyboard::key::Named::ArrowDown;
use iced::keyboard::key::Named::ArrowLeft;
use iced::keyboard::key::Named::ArrowRight;
use iced::keyboard::key::Named::ArrowUp;
use iced::keyboard::key::Named::Enter;
use iced::keyboard::key::Named::Escape;
use iced::widget::Space;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text_input;
use reqwest::blocking::Client;
use strum_macros::EnumIter;

use crate::app;
use crate::config::Config;
use crate::options_page;
use crate::options_page::OptionsPage;
use crate::page::AppUpdate;
use crate::presets::help_text;
use crate::presets::transparent_button_sized;
use crate::search_page::SEARCH_BAR_ID;
use crate::search_page::SearchPage;
use crate::{list_query_state::ListQueryState, page::Page, presets::square_box};

#[derive(Debug, Clone)]
pub enum Message {
    Select(Selection),
    KeyPressed(Key),
}

#[derive(EnumIter, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Selection {
    Search,
    Options,
    Exit,
}

impl Selection {
    pub fn next(&self) -> Self {
        match self {
            Self::Search => Self::Options,
            Self::Options => Self::Exit,
            Self::Exit => Self::Exit,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Search => Self::Search,
            Self::Options => Self::Search,
            Self::Exit => Self::Options,
        }
    }
}

impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Search => "Buscar",
                Self::Options => "Opciones",
                Self::Exit => "Salir",
            }
        )
    }
}

pub struct MainMenuPage {
    pub config: Config,
    pub client: Client,
    pub theme: Theme,
    pub selection: Selection,
    pub anime_list: ListQueryState,
}

impl Page for MainMenuPage {
    fn view(&self) -> iced::Element<'_, app::Message> {
        square_box(column![
            Space::with_height(Length::Fill),
            container(
                text("Ani-Link")
                    .font(Font {
                        weight: iced::font::Weight::Black,
                        ..Font::DEFAULT
                    })
                    .size(100)
            )
            .style(|theme: &iced::Theme| container::Style {
                text_color: Some(theme.palette().primary),
                ..Default::default()
            })
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            container(
                transparent_button_sized("Buscar", matches!(self.selection, Selection::Search), 24)
                    .on_press(app::Message::MainMenu(Message::Select(Selection::Search)))
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            container(
                transparent_button_sized(
                    "Opciones",
                    matches!(self.selection, Selection::Options),
                    24
                )
                .on_press(app::Message::MainMenu(Message::Select(Selection::Options)))
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            container(
                transparent_button_sized("Salir", matches!(self.selection, Selection::Exit), 24)
                    .on_press(app::Message::MainMenu(Message::Select(Selection::Exit)))
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            Space::with_height(Length::Fill),
            container(row![
                text("Subir:"),
                help_text(" ↑ K "),
                text(" Bajar:"),
                help_text(" ↓ J "),
                text(" Confirmar:"),
                help_text(" → L Enter "),
                text(" Salir:"),
                help_text(" ← H Esc"),
            ])
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            Space::with_height(Length::Fixed(3.0)),
        ])
        .into()
    }

    fn update(&mut self, message: app::Message) -> AppUpdate {
        let mut change_selection = |selection| -> AppUpdate {
            match selection {
                Selection::Search => {
                    let anime_list = mem::take(&mut self.anime_list);

                    let anime_list = match anime_list {
                        ListQueryState::Obtaining(..) => anime_list.get(),
                        ListQueryState::Obtained(..) => anime_list,
                    };

                    let ListQueryState::Obtained(anime_list) = anime_list else {
                        panic!("Should not happen");
                    };

                    let filtered_list = anime_list.clone();

                    AppUpdate::Both((
                        Box::new(SearchPage {
                            config: mem::take(&mut self.config),
                            client: mem::take(&mut self.client),
                            theme: mem::take(&mut self.theme),
                            anime_list,
                            query: String::new(),
                            selected: 0,
                            filtered_list,
                        }),
                        text_input::focus(text_input::Id::new(SEARCH_BAR_ID)),
                    ))
                }
                Selection::Options => AppUpdate::Page(Box::new(OptionsPage {
                    old_config: self.config.clone(),
                    config: mem::take(&mut self.config),
                    client: mem::take(&mut self.client),
                    theme: mem::take(&mut self.theme),
                    anime_list: mem::take(&mut self.anime_list),
                    selection: options_page::Options::Scraper,
                })),
                Selection::Exit => exit(0),
            }
        };

        if let app::Message::MainMenu(message) = message {
            match message {
                Message::Select(selection) => change_selection(selection),
                Message::KeyPressed(key) => match key.as_ref() {
                    Key::Character("j") | Key::Named(ArrowDown) => {
                        self.selection = self.selection.next();
                        AppUpdate::None
                    }
                    Key::Character("k") | Key::Named(ArrowUp) => {
                        self.selection = self.selection.prev();
                        AppUpdate::None
                    }
                    Key::Character("l") | Key::Named(Enter) | Key::Named(ArrowRight) => {
                        change_selection(self.selection)
                    }
                    Key::Character("h") | Key::Named(Escape) | Key::Named(ArrowLeft) => exit(0),
                    _ => AppUpdate::None,
                },
            }
        } else {
            panic!("main menu event handler called for non main menu event")
        }
    }

    fn subscription(&self) -> iced::Subscription<app::Message> {
        event::listen_with(move |event, status, _| match (event, status) {
            (Event::Keyboard(KeyPressed { key, .. }), Status::Ignored) => {
                Some(app::Message::MainMenu(Message::KeyPressed(key)))
            }
            _ => None,
        })
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}
