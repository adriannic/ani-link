use std::{fmt, mem, process::exit, sync::atomic::Ordering};

use iced::{
    Event, Font, Length, Subscription, Theme,
    alignment::Horizontal,
    event::{self, Status},
    keyboard::{
        Event::KeyPressed,
        Key,
        key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter, Escape},
    },
    time::{self, Duration},
    widget::{Space, column, container, row, text, text_input},
};
use reqwest::Client;
use strum_macros::EnumIter;

use crate::{
    app,
    config::Config,
    list_query_state::ListQueryState,
    options_page::{self, OptionsPage},
    page::{AppUpdate, Page},
    presets::{help_text, square_box, transparent_button},
    search_page::{SEARCH_BAR_ID, SearchPage},
};

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
    pub fn next(self) -> Self {
        match self {
            Self::Search => Self::Options,
            Self::Options | Self::Exit => Self::Exit,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Search | Self::Options => Self::Search,
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
    pub waiting: bool,
}

impl Page for MainMenuPage {
    fn view(&self) -> iced::Element<'_, app::Message> {
        let progress = match &self.anime_list {
            ListQueryState::Obtaining(_, progress) | ListQueryState::Obtained(_, progress) => {
                progress.clone()
            }
        }
        .load(Ordering::SeqCst);

        let total = self.config.scraper.pages();

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
                if progress == total {
                    transparent_button("Buscar", matches!(self.selection, Selection::Search))
                } else {
                    transparent_button(
                        &format!("Buscar ({}%)", 100 * progress / total),
                        matches!(self.selection, Selection::Search),
                    )
                }
                .on_press(app::Message::MainMenu(Message::Select(Selection::Search)))
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            container(
                transparent_button("Opciones", matches!(self.selection, Selection::Options),)
                    .on_press(app::Message::MainMenu(Message::Select(Selection::Options)))
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            container(
                transparent_button("Salir", matches!(self.selection, Selection::Exit))
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
                    let progress = match &self.anime_list {
                        ListQueryState::Obtaining(_, progress)
                        | ListQueryState::Obtained(_, progress) => progress.clone(),
                    };

                    if progress.load(Ordering::SeqCst) != self.config.scraper.pages() {
                        self.waiting = true;
                        return AppUpdate::None;
                    }

                    let anime_list = mem::take(&mut self.anime_list);

                    let anime_list = match anime_list {
                        ListQueryState::Obtaining(..) => anime_list.get(),
                        ListQueryState::Obtained(..) => anime_list,
                    };

                    let ListQueryState::Obtained(anime_list, _) = anime_list else {
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
                    Key::Character("l") | Key::Named(Enter | ArrowRight) => {
                        change_selection(self.selection)
                    }
                    Key::Character("h") | Key::Named(Escape | ArrowLeft) => exit(0),
                    _ => AppUpdate::None,
                },
            }
        } else if matches!(message, app::Message::UpdateProgress) {
            let progress = match &self.anime_list {
                ListQueryState::Obtaining(_, progress) | ListQueryState::Obtained(_, progress) => {
                    progress.clone()
                }
            };

            if self.waiting && progress.load(Ordering::SeqCst) == self.config.scraper.pages() {
                let anime_list = mem::take(&mut self.anime_list);

                let anime_list = match anime_list {
                    ListQueryState::Obtaining(..) => anime_list.get(),
                    ListQueryState::Obtained(..) => anime_list,
                };

                let ListQueryState::Obtained(anime_list, _) = anime_list else {
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
            } else {
                AppUpdate::None
            }
        } else {
            panic!("main menu event handler called for non main menu event")
        }
    }

    fn subscription(&self) -> iced::Subscription<app::Message> {
        let mut subscriptions =
            vec![time::every(Duration::from_millis(100)).map(|_| app::Message::UpdateProgress)];

        if !self.waiting {
            subscriptions.push(event::listen_with(move |event, status, _| {
                match (event, status) {
                    (Event::Keyboard(KeyPressed { key, .. }), Status::Ignored) => {
                        Some(app::Message::MainMenu(Message::KeyPressed(key)))
                    }
                    _ => None,
                }
            }));
        }
        Subscription::batch(subscriptions)
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}
