use main_menu::{MainMenuSelection, draw_main_menu, handle_events_main_menu};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListDirection, ListState};
use reqwest::blocking::Client;
use search::{draw_search, handle_events_search};
use std::error::Error;
use strum::IntoEnumIterator;

use crate::episodes::{draw_episodes, handle_events_episodes};
use crate::options::{draw_options, handle_events_options};
use crate::config::Config;
use crate::{main_menu, search};
use crate::menu_state::{ListQueryState, MenuState};

pub struct App {
    pub running: bool,
    pub config: Config,
    pub menu_state: MenuState,
    pub main_menu_selection: ListState,
    pub terminal: DefaultTerminal,
    pub client: Client,
}

impl App {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let config: Config = Config::init()?;
        let term = ratatui::init();
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            )
            .cookie_store(true)
            .build()
            .expect("Couldn't build client");

        let scraper = config.scraper;

        Ok(Self {
            running: true,
            config,
            menu_state: MenuState::MainMenu {
                anime_list: ListQueryState::spawn(scraper, client.clone()),
                should_draw_popup: false,
            },
            main_menu_selection: ListState::default().with_selected(Some(0)),
            terminal: term,
            client,
        })
    }

    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        while self.running {
            self.draw()?;
            self.handle_events()?;
        }

        Ok(())
    }

    pub(crate) fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        self.terminal.draw(|frame| {
            frame.render_widget(Clear, frame.area());

            // Divide areas
            let horizontal = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]);

            let [menu_selector_area, content_area] = horizontal.areas(frame.area());

            match self.menu_state {
                MenuState::MainMenu {
                    should_draw_popup: searching,
                    ..
                } => draw_main_menu(frame, content_area, searching),
                MenuState::Search {
                    ref anime_list,
                    search_state,
                    ref query,
                    ref mut anime_state,
                    ..
                } => {
                    draw_search(
                        frame,
                        content_area,
                        anime_list,
                        search_state,
                        query,
                        anime_state,
                    );
                }
                MenuState::Options { ref mut state, .. } => {
                    draw_options(frame, content_area, &self.config, state)
                }
                MenuState::Episodes {
                    ref mut state,
                    ref anime,
                    ref episodes,
                    ..
                } => draw_episodes(frame, content_area, anime, state, episodes),
            }

            // Render Main menu option list
            let list_title = Line::from(" Menú principal ".bold().white()).centered();

            let list_block = Block::bordered()
                .title(list_title)
                .border_set(border::THICK)
                .border_style(Style::new().green());

            let items = MainMenuSelection::iter().map(|scraper| scraper.to_string());

            let mut main_menu_list = List::new(items)
                .block(list_block)
                .highlight_symbol("> ")
                .highlight_style(Style::new().bold())
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            if !matches!(self.menu_state, MenuState::MainMenu { .. }) {
                main_menu_list = main_menu_list.style(Style::new().gray());
            }

            frame.render_stateful_widget(
                main_menu_list,
                menu_selector_area,
                &mut self.main_menu_selection,
            );
        })?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        match self.menu_state {
            MenuState::MainMenu { .. } => handle_events_main_menu(self),
            MenuState::Search { .. } => handle_events_search(self),
            MenuState::Episodes { .. } => handle_events_episodes(self),
            MenuState::Options { .. } => handle_events_options(self),
        }

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
