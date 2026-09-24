use std::{fmt, mem, process::exit, sync::atomic::Ordering};

use iced::{
    Event, Font, Length, Subscription, Task,
    alignment::Horizontal,
    event::{self, Status},
    keyboard::{
        Event::KeyPressed,
        Key,
        key::Named::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter, Escape},
    },
    never,
    time::{self, Duration},
    widget::{Id, Space, column, container, operation::focus, rich_text, span, text},
};
use reqwest::Client;
use rust_i18n::t;
use strum_macros::EnumIter;

use crate::{
    app,
    config::Config,
    list_query_state::ListQueryState,
    options_page::{self, OptionsPage},
    page::{AppUpdate, Page},
    presets::{square_box, transparent_button},
    search_page::{SEARCH_BAR_ID, SearchPage},
};

#[derive(Debug, Clone)]
pub enum Message {
    Select(Selection),
    KeyPressed(Key),
}

#[derive(EnumIter, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Selection {
    Anime,
    Options,
    Exit,
}

impl Selection {
    pub const fn next(self) -> Self {
        match self {
            Self::Anime => Self::Options,
            Self::Options | Self::Exit => Self::Exit,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Anime | Self::Options => Self::Anime,
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
                Self::Anime => t!("anime"),
                Self::Options => t!("options"),
                Self::Exit => t!("exit"),
            }
        )
    }
}

pub struct MainMenuPage {
    pub config: Config,
    pub client: Client,
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
        .load(Ordering::Relaxed);

        let total = self.config.scraper.pages();

        square_box(column![
            Space::new().height(Length::Fill),
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
                if self.waiting {
                    transparent_button(
                        &format!("{} ({progress}/{total})", t!("fetching")),
                        matches!(self.selection, Selection::Anime),
                    )
                } else {
                    transparent_button(&t!("anime"), matches!(self.selection, Selection::Anime))
                }
                .on_press(app::Message::MainMenu(Message::Select(Selection::Anime)))
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            container(
                transparent_button(&t!("options"), matches!(self.selection, Selection::Options),)
                    .on_press(app::Message::MainMenu(Message::Select(Selection::Options)))
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            container(
                transparent_button(&t!("exit"), matches!(self.selection, Selection::Exit))
                    .on_press(app::Message::MainMenu(Message::Select(Selection::Exit)))
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            Space::new().height(Length::Fill),
            container(
                text(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .color(self.config.theme().palette().primary)
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            container(
                rich_text![
                    span(format!("{}:", t!("up"))).color(self.config.theme().palette().text),
                    span(" ↑ K ").color(self.config.theme().palette().primary),
                    span(format!(" {}:", t!("down"))).color(self.config.theme().palette().text),
                    span(" ↓ J ").color(self.config.theme().palette().primary),
                    span(format!(" {}:", t!("confirm"))).color(self.config.theme().palette().text),
                    span(" → L Enter ").color(self.config.theme().palette().primary),
                    span(format!(" {}:", t!("exit"))).color(self.config.theme().palette().text),
                    span(" ← H Esc").color(self.config.theme().palette().primary),
                ]
                .on_link_click(never)
            )
            .align_x(Horizontal::Center)
            .width(Length::Fill),
            Space::new().height(Length::Fixed(3.0)),
        ])
        .into()
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: app::Message) -> AppUpdate {
        let mut change_selection = |selection| -> AppUpdate {
            match selection {
                Selection::Anime => {
                    let progress = match &self.anime_list {
                        ListQueryState::Obtaining(_, progress)
                        | ListQueryState::Obtained(_, progress) => progress.clone(),
                    };

                    if progress.load(Ordering::Relaxed) != self.config.scraper.pages() {
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

                    let mut page = SearchPage {
                        config: mem::take(&mut self.config),
                        client: mem::take(&mut self.client),
                        anime_list,
                        query: String::new(),
                        selected: 0,
                        filtered_list,
                        image: None,
                    };

                    let image_task = page.fuzzy();

                    AppUpdate::Both((
                        Box::new(page),
                        Task::batch(vec![focus(Id::new(SEARCH_BAR_ID)), image_task]),
                    ))
                }
                Selection::Options => AppUpdate::Page(Box::new(OptionsPage {
                    old_config: self.config.clone(),
                    config: mem::take(&mut self.config),
                    client: mem::take(&mut self.client),
                    anime_list: mem::take(&mut self.anime_list),
                    selection: options_page::Options::default(),
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
        } else if matches!(message, app::Message::Update) {
            let progress = match &self.anime_list {
                ListQueryState::Obtaining(_, progress) | ListQueryState::Obtained(_, progress) => {
                    progress.clone()
                }
            };

            if self.waiting && progress.load(Ordering::Relaxed) == self.config.scraper.pages() {
                let anime_list = mem::take(&mut self.anime_list);

                let anime_list = match anime_list {
                    ListQueryState::Obtaining(..) => anime_list.get(),
                    ListQueryState::Obtained(..) => anime_list,
                };

                let ListQueryState::Obtained(anime_list, _) = anime_list else {
                    panic!("Should not happen");
                };

                let filtered_list = anime_list.clone();

                let mut page = SearchPage {
                    config: mem::take(&mut self.config),
                    client: mem::take(&mut self.client),
                    anime_list,
                    query: String::new(),
                    selected: 0,
                    filtered_list,
                    image: None,
                };

                let image_task = page.fuzzy();

                AppUpdate::Both((
                    Box::new(page),
                    Task::batch(vec![focus(Id::new(SEARCH_BAR_ID)), image_task]),
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
            vec![time::every(Duration::from_millis(100)).map(|_| app::Message::Update)];

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
        self.config.theme()
    }
}
