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
use std::process::Command;
use std::thread::{self, JoinHandle};
use strum::IntoEnumIterator;

use crate::config::Config;
use crate::menu_state::MenuState;
use crate::scraper::anime::Anime;
use crate::scraper::animeav1scraper::AnimeAv1Scraper;
use crate::scraper::animeflvscraper::AnimeFlvScraper;
use crate::scraper::{Scraper, ScraperImpl};

pub struct App {
    pub running: bool,
    pub scraper: ScraperImpl,
    pub menu_state: MenuState,
    pub main_menu_selection: ListState,
    pub terminal: DefaultTerminal,
    pub anime_thread: Option<JoinHandle<Vec<Anime>>>,
    pub client: Client,
}

impl App {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let config: Config = Config::init()?;
        let term = ratatui::init();
        let scraper = config.scraper;
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            )
            .cookie_store(true)
            .build()
            .expect("Couldn't build client");

        let thread_client = client.clone();

        Ok(Self {
            running: true,
            scraper: config.scraper,
            menu_state: MenuState::MainMenu { searching: false },
            main_menu_selection: ListState::default().with_selected(Some(0)),
            terminal: term,
            anime_thread: Some(thread::spawn(move || {
                Command::new("notify-send")
                    .arg("anime-thread")
                    .arg("Starting")
                    .output()
                    .unwrap();

                let result = match scraper {
                    ScraperImpl::AnimeAv1Scraper => AnimeAv1Scraper::try_search(&thread_client)
                        .expect("Couldn't retrieve the list of animes"),
                    ScraperImpl::AnimeFlvScraper => AnimeFlvScraper::try_search(&thread_client)
                        .expect("Couldn't retrieve the list of animes"),
                };

                Command::new("notify-send")
                    .arg("anime-thread")
                    .arg("Finished")
                    .output()
                    .unwrap();

                result
            })),
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

    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        self.terminal.draw(|frame| {
            frame.render_widget(Clear, frame.area());

            // Divide areas
            let horizontal = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]);

            let [menu_selector_area, content_area] = horizontal.areas(frame.area());

            match self.menu_state {
                MenuState::MainMenu { searching } => draw_main_menu(frame, content_area, searching),
                MenuState::Search {
                    ref anime_list,
                    search_state: state,
                    ref query,
                    ref mut anime_state,
                    ..
                } => {
                    draw_search(frame, content_area, anime_list, state, query, anime_state);
                }
                _ => todo!(),
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
            _ => todo!(),
        }

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

mod main_menu {
    use std::fmt;

    use ratatui::{
        Frame,
        crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        layout::{Constraint, Flex, Layout, Rect},
        style::{Style, Stylize},
        symbols::border,
        text::Line,
        widgets::{Block, Clear, ListState, Paragraph},
    };
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    use crate::menu_state::MenuState;

    use super::{App, search::SearchState};

    #[derive(EnumIter, PartialEq, Eq)]
    pub enum MainMenuSelection {
        Search,
        Options,
        Exit,
    }

    impl fmt::Display for MainMenuSelection {
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

    fn get_popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    pub fn draw_main_menu(frame: &mut Frame, content_area: Rect, searching: bool) {
        let vertical = ratatui::layout::Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Fill(1),
        ]);

        let [_, banner_area, _] = vertical.areas(content_area);

        // Render banner
        let right_banner = Paragraph::new(
            "
 █████╗ ███╗   ██╗██╗      ██╗     ██╗███╗   ██╗██╗  ██╗
██╔══██╗████╗  ██║██║      ██║     ██║████╗  ██║██║ ██╔╝
███████║██╔██╗ ██║██║█████╗██║     ██║██╔██╗ ██║█████╔╝ 
██╔══██║██║╚██╗██║██║╚════╝██║     ██║██║╚██╗██║██╔═██╗ 
██║  ██║██║ ╚████║██║      ███████╗██║██║ ╚████║██║  ██╗
╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝      ╚══════╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝
",
        )
        .block(Block::new())
        .bold()
        .blue()
        .centered();

        frame.render_widget(right_banner, banner_area);

        // Render right block
        let right_title = Line::from(
            format!(" Ani-link v{} ", env!("CARGO_PKG_VERSION"))
                .bold()
                .white(),
        )
        .centered();

        let right_instructions = Line::from(vec![
            " Subir:".white(),
            " ↑ K ".blue().bold(),
            " Bajar:".white(),
            " ↓ J ".blue().bold(),
            " Confirmar:".white(),
            " → L Enter ".blue().bold(),
            " Salir:".white(),
            " ← H Esc ".blue().bold(),
        ]);

        let right_block = Block::bordered()
            .title(right_title)
            .title_bottom(right_instructions.centered())
            .border_set(border::THICK)
            .border_style(Style::new().green());

        frame.render_widget(right_block, content_area);

        if searching {
            let popup = Paragraph::new("\nCargando lista de animes...")
                .block(
                    Block::bordered()
                        .title(Line::from(" Cargando... ").centered().white())
                        .border_set(border::THICK)
                        .border_style(Style::new().green())
                        .on_black(),
                )
                .bold()
                .centered();

            let popup_area = get_popup_area(frame.area(), 20, 12);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(popup, popup_area);
        }
    }

    pub fn handle_events_main_menu(app: &mut App) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read().expect("Couldn't read event from main menu")
        {
            match code {
                KeyCode::Up | KeyCode::Char('k') => app.main_menu_selection.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => app.main_menu_selection.select_next(),
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                    if let Some(i) = app.main_menu_selection.selected() {
                        let option = MainMenuSelection::iter().nth(i).unwrap();
                        match option {
                            MainMenuSelection::Search => {
                                if let MenuState::MainMenu { searching } = &mut app.menu_state {
                                    *searching = true;
                                    app.draw().unwrap();
                                }
                                if let MenuState::MainMenu { searching } = &mut app.menu_state {
                                    *searching = false;
                                    let anime_list = app
                                        .anime_thread
                                        .take()
                                        .expect("Thread missing")
                                        .join()
                                        .expect("Thread couldn't be joined");

                                    let filtered_list = anime_list.clone();

                                    app.menu_state = MenuState::Search {
                                        anime_list,
                                        search_state: SearchState::Searching,
                                        query: String::new(),
                                        anime_state: ListState::default().with_selected(Some(0)),
                                        filtered_list,
                                    }
                                }
                            }
                            MainMenuSelection::Options => app.menu_state = MenuState::Options,
                            MainMenuSelection::Exit => app.running = false,
                        }
                    }
                }
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => app.running = false,
                _ => {}
            }
        }
    }
}

pub mod search {
    use fuzzy_matcher::clangd::fuzzy_match;
    use ratatui::{
        Frame,
        crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        layout::{Constraint, Layout, Rect},
        style::{Style, Stylize},
        symbols::border,
        text::Line,
        widgets::{Block, List, ListDirection, ListState, Paragraph},
    };
    use rayon::prelude::*;

    use crate::{menu_state::MenuState, scraper::anime::Anime};

    use super::App;

    const MAX_QUERY: usize = 80;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
    pub enum SearchState {
        Searching,
        Choosing,
    }

    pub fn draw_search(
        frame: &mut Frame,
        content_area: Rect,
        anime_list: &[Anime],
        search_state: SearchState,
        query: &str,
        anime_state: &mut ListState,
    ) {
        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]);

        let [search_area, list_area] = vertical.areas(content_area);

        // Render search block
        let search_title = Line::from(" Buscar ".bold().white()).centered();

        let search_instructions = Line::from(vec![
            " Atrás:".white(),
            " Esc ".blue().bold(),
            " Confirmar:".white(),
            " Enter ".blue().bold(),
        ]);

        let mut search_block = Block::bordered()
            .title(search_title)
            .border_set(border::THICK)
            .border_style(Style::new().green());

        if search_state == SearchState::Searching {
            search_block = search_block.title_bottom(search_instructions.centered());
        }

        let mut search_text = Paragraph::new(format!(" Nombre: {query}"))
            .block(search_block)
            .bold();

        if search_state == SearchState::Choosing {
            search_text = search_text.style(Style::new().gray());
        }

        frame.render_widget(search_text, search_area);

        // Render list block
        let list_title = Line::from(" Animes ".bold().white()).centered();

        let list_instructions = Line::from(vec![
            " Subir:".white(),
            " ↑ K ".blue().bold(),
            " Bajar:".white(),
            " ↓ J ".blue().bold(),
            " Confirmar:".white(),
            " → L Enter ".blue().bold(),
            " Atrás:".white(),
            " ← H Esc ".blue().bold(),
        ]);

        let mut list_block = Block::bordered()
            .title(list_title)
            .border_set(border::THICK)
            .border_style(Style::new().green());

        if search_state == SearchState::Choosing {
            list_block = list_block.title_bottom(list_instructions.centered());
        }

        let mut filtered_anime = anime_list
            .iter()
            .par_bridge()
            .filter_map(|anime| {
                if anime
                    .names
                    .iter()
                    .any(|name| fuzzy_match(name, query).is_some())
                {
                    Some(&anime.names[0])
                } else {
                    None
                }
            })
            .cloned()
            .collect::<Vec<_>>();

        filtered_anime.par_sort();

        let mut anime_list = List::new(filtered_anime)
            .block(list_block)
            .highlight_symbol("> ")
            .highlight_style(Style::new().bold())
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        if search_state == SearchState::Searching {
            anime_list = anime_list.style(Style::new().gray());
        }

        frame.render_stateful_widget(anime_list, list_area, anime_state);
    }

    pub fn handle_events_search(app: &mut App) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read().expect("Couldn't read event from search menu")
        {
            let MenuState::Search {
                search_state,
                query,
                anime_state,
                filtered_list,
                ..
            } = &mut app.menu_state
            else {
                panic!("Invalid app state in search menu");
            };

            match search_state {
                SearchState::Searching => match code {
                    KeyCode::Esc => {
                        app.menu_state = MenuState::MainMenu { searching: false };
                    }
                    KeyCode::Enter => *search_state = SearchState::Choosing,
                    KeyCode::Backspace => {
                        if !query.is_empty() {
                            anime_state.select_first();
                            query.pop();
                            if query.is_empty() {
                                anime_state.select(None);
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if query.len() < MAX_QUERY {
                            query.push(c);
                            anime_state.select_first();
                        }
                    }
                    _ => {}
                },
                SearchState::Choosing => match code {
                    KeyCode::Up | KeyCode::Char('k') => anime_state.select_previous(),
                    KeyCode::Down | KeyCode::Char('j') => anime_state.select_next(),
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                        if let Some(i) = anime_state.selected() {
                            if let Some(anime) = filtered_list.get(i) {
                                app.menu_state = MenuState::Episodes {
                                    anime: anime.clone(),
                                };
                            }
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                        *search_state = SearchState::Searching;
                        anime_state.select_first();
                    }
                    _ => {}
                },
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn hello() {
        println!("Hello world!");
    }
}
