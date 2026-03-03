use crate::list_query_state::ListQueryState;
use crate::main_menu_page::{self, MainMenuPage};
use crate::options_page;
use crate::page::Page;
use iced::{Font, Settings};
use reqwest::blocking::Client;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Message {
    MainMenu(main_menu_page::Message),
    Options(options_page::Message),
}

pub struct App {
    pub page: Box<dyn Page>,
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
        let theme = config.theme.into();

        Self {
            page: Box::new(MainMenuPage {
                config,
                client: client.clone(),
                theme,
                selection: main_menu_page::Selection::Search,
                anime_list: ListQueryState::spawn(scraper, client),
            }),
        }
    }
}

impl App {
    pub fn run() -> iced::Result {
        iced::application("Ani-Link", App::update, App::view)
            .theme(|app| app.page.theme())
            .subscription(App::subscription)
            .transparent(true)
            .antialiasing(true)
            .settings(Settings {
                default_text_size: 14.into(),
                ..Default::default()
            })
            .font(include_bytes!("../assets/font.ttf"))
            .default_font(Font {
                weight: iced::font::Weight::Normal,
                ..Font::with_name("FiraCode Nerd Font Mono")
            })
            .run()
    }

    pub(crate) fn view(&self) -> iced::Element<'_, Message> {
        self.page.view()
    }

    fn update(&mut self, message: Message) {
        let page = self.page.update(message);
        if let Some(p) = page {
            self.page = p;
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        self.page.subscription()
    }
}
