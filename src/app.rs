use std::sync::LazyLock;

use crate::episodes::draw_episodes;
use crate::main_menu::{MainMenuSelection, draw_main_menu, handle_events_main_menu};
use crate::options::{OptionEvent, draw_options};
use crate::presets::{square_box, transparent_button};
use crate::search::draw_search;
use iced::theme::Palette;
use iced::widget::{column, row};
use iced::{Color, Font, Length, Settings, Theme, color};
use reqwest::blocking::Client;

use crate::config::Config;
use crate::menu_state::{ListQueryState, MenuState};

static CUSTOM_THEME: LazyLock<Theme> = LazyLock::new(|| {
    iced::Theme::custom(
        "custom".into(),
        Palette {
            background: Color::from_rgba(
                0x03 as f32 / 255.0,
                0x04 as f32 / 255.0,
                0x0D as f32 / 255.0,
                0.75,
            ),
            text: color!(0xD9CBD2),
            primary: color!(0xA49029),
            success: color!(0x00FF00),
            danger: color!(0xFF0000),
        },
    )
});

#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    MainMenu(MainMenuSelection),
    Options(OptionEvent),
}

pub struct App {
    pub running: bool,
    pub config: Config,
    pub menu_state: MenuState,
    pub main_menu_selection: MainMenuSelection,
    pub client: Client,
    pub theme: Theme,
}

impl Default for App {
    fn default() -> Self {
        let config: Config = Config::init().expect("Couldn't initialize config");
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            )
            .cookie_store(true)
            .build()
            .expect("Couldn't build client");

        let scraper = config.scraper;

        Self {
            running: true,
            config,
            menu_state: MenuState::MainMenu {
                anime_list: ListQueryState::spawn(scraper, client.clone()),
                should_draw_popup: false,
            },
            main_menu_selection: MainMenuSelection::Search,
            client,
            theme: CUSTOM_THEME.clone(),
        }
    }
}

impl App {
    pub fn run() -> iced::Result {
        iced::application("Ani-Link", App::update, App::view)
            .theme(App::theme)
            .transparent(true)
            .antialiasing(true)
            .settings(Settings {
                default_text_size: 14.into(),
                ..Default::default()
            })
            .font(include_bytes!("../assets/font.ttf"))
            .default_font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::with_name("FiraCode Nerd Font Mono")
            })
            .run()
    }

    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }

    pub(crate) fn view(&self) -> iced::Element<'_, AppEvent> {
        row![
            square_box(
                column![
                    transparent_button("Buscar", self.theme.palette(), true)
                        .width(Length::Fill)
                        .on_press(AppEvent::MainMenu(MainMenuSelection::Search)),
                    transparent_button("Opciones", self.theme.palette(), false)
                        .width(Length::Fill)
                        .on_press(AppEvent::MainMenu(MainMenuSelection::Options)),
                    transparent_button("Salir", self.theme.palette(), false)
                        .width(Length::Fill)
                        .on_press(AppEvent::MainMenu(MainMenuSelection::Exit)),
                ]
                .padding(6),
                self.theme.palette()
            )
            .width(Length::Fixed(150.0))
            .height(Length::Fill),
            match self.menu_state {
                MenuState::MainMenu {
                    should_draw_popup: searching,
                    ..
                } => draw_main_menu(self, searching),
                MenuState::Search {
                    ref filtered_list,
                    ref query,
                    ..
                } => {
                    draw_search(self, query, filtered_list)
                }
                MenuState::Options { .. } => {
                    draw_options(self)
                }
                MenuState::Episodes { ref episodes, .. } => draw_episodes(episodes),
            }
        ]
        .padding(3)
        .into()

        // // Render Main menu option list
        // let list_title = Line::from(" Menú principal ".bold().white()).centered();
        //
        // let list_block = Block::bordered()
        //     .title(list_title)
        //     .border_set(border::THICK)
        //     .border_style(Style::new().green());
        //
        // let items = MainMenuSelection::iter().map(|scraper| scraper.to_string());
        //
        // let mut main_menu_list = List::new(items)
        //     .block(list_block)
        //     .highlight_symbol("> ")
        //     .highlight_style(Style::new().bold())
        //     .repeat_highlight_symbol(true)
        //     .direction(ListDirection::TopToBottom);
        //
        // if !matches!(self.menu_state, MenuState::MainMenu { .. }) {
        //     main_menu_list = main_menu_list.style(Style::new().gray());
        // }
        //
        // frame.render_stateful_widget(
        //     main_menu_list,
        //     menu_selector_area,
        //     &mut self.main_menu_selection,
        // );
    }

    fn update(&mut self, message: AppEvent) {
        match self.menu_state {
            MenuState::MainMenu { .. } => handle_events_main_menu(self),
            // MenuState::Search { .. } => handle_events_search(self),
            // MenuState::Episodes { .. } => handle_events_episodes(self),
            // MenuState::Options { .. } => handle_events_options(self),
            _ => todo!(),
        }
    }
}
