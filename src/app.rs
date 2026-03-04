use iced::{Font, Settings, Task};
use reqwest::blocking::Client;

use crate::{
    config::Config,
    list_query_state::ListQueryState,
    main_menu_page::{self, MainMenuPage},
    options_page,
    page::{AppUpdate, Page},
    search_page,
};

#[derive(Debug, Clone)]
pub enum Message {
    MainMenu(main_menu_page::Message),
    Options(options_page::Message),
    Search(search_page::Message),
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

        let anime_list = ListQueryState::spawn(scraper, client.clone());

        Self {
            page: Box::new(MainMenuPage {
                config,
                client,
                theme,
                selection: main_menu_page::Selection::Search,
                anime_list,
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

    fn update(&mut self, message: Message) -> Task<Message> {
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

    fn subscription(&self) -> iced::Subscription<Message> {
        self.page.subscription()
    }
}
