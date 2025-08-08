use std::{fmt, mem};

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

use crate::{app::App, menu_state::{ListQueryState, MenuState}};

use super::{search::SearchState};

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
                            let MenuState::MainMenu {
                                anime_list,
                                should_draw_popup,
                            } = &mut app.menu_state
                            else {
                                panic!("Invalid app state in main menu")
                            };

                            let anime_list = mem::take(anime_list);

                            let anime_list = match anime_list {
                                ListQueryState::Obtaining(..) => {
                                    *should_draw_popup = true;
                                    app.draw().unwrap();
                                    anime_list.get()
                                }
                                ListQueryState::Obtained(..) => anime_list,
                                _ => panic!("Invalid anime_list state"),
                            };

                            let ListQueryState::Obtained(anime_list) = anime_list else {
                                panic!("Should not happen")
                            };

                            let filtered_list = anime_list.clone();

                            app.menu_state = MenuState::Search {
                                anime_list,
                                search_state: SearchState::Searching,
                                query: String::new(),
                                anime_state: ListState::default().with_selected(Some(0)),
                                filtered_list,
                            }
                        }
                        MainMenuSelection::Options => {
                            let MenuState::MainMenu { anime_list, .. } = &mut app.menu_state
                            else {
                                panic!("Invalid app state in main menu")
                            };

                            app.menu_state = MenuState::Options {
                                anime_list: mem::take(anime_list),
                                old_config: app.config.clone(),
                                state: ListState::default().with_selected(Some(0)),
                            }
                        }
                        MainMenuSelection::Exit => app.running = false,
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => app.running = false,
            _ => {}
        }
    }
}

