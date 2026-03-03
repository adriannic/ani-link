use std::{fmt, mem, process::exit};

use iced::{
    Length,
    widget::{container, svg},
};
use strum_macros::EnumIter;

use crate::{
    app::{App, Message},
    options::Options,
    presets::square_box,
};

use super::search::SearchState;

pub fn draw_main_menu<'a>(app: &'a App, searching: bool) -> iced::Element<'a, Message> {
    square_box(
        container(
            svg("assets/logo.svg")
                .width(Length::Fixed(800.0))
                .height(Length::Fill),
        )
        .center(Length::Fill)
        .padding(100),
        app.theme.palette(),
    )
    .into()
}

pub fn handle_events_main_menu(app: &mut App, message: Message) {
    let page = match &mut app.page {
        Page::Search { anime_list, .. } => {
            app.page = Page::MainMenu {
                anime_list: ListQueryState::Obtained(mem::take(anime_list)),
                should_draw_popup: false,
            };

            &mut app.page
        }
        Page::Episodes { anime_list, .. } => {
            app.page = Page::MainMenu {
                anime_list: ListQueryState::Obtained(mem::take(anime_list)),
                should_draw_popup: false,
            };

            &mut app.page
        }
        Page::Options { anime_list, .. } => {
            app.page = Page::MainMenu {
                anime_list: mem::take(anime_list),
                should_draw_popup: false,
            };

            &mut app.page
        }
        Page::MainMenu { .. } => &mut app.page,
    };

    let Page::MainMenu {
        anime_list,
        should_draw_popup,
    } = page
    else {
        panic!("Should not happen");
    };

    match message {
        Message::MainMenu(selection) => match selection {
            MainMenuSelection::Search => {
                let anime_list = mem::take(anime_list);

                let anime_list = match anime_list {
                    ListQueryState::Obtaining(..) => {
                        *should_draw_popup = true;
                        draw_main_menu(app, true);
                        anime_list.get()
                    }
                    ListQueryState::Obtained(..) => anime_list,
                    _ => panic!("Invalid anime_list state"),
                };

                let ListQueryState::Obtained(anime_list) = anime_list else {
                    panic!("Should not happen");
                };

                let filtered_list = anime_list.clone();

                app.page = Page::Search {
                    anime_list,
                    search_state: SearchState::Searching,
                    query: String::new(),
                    anime_state: 0,
                    filtered_list,
                };

                app.main_menu_selection = MainMenuSelection::Search;
            }
            MainMenuSelection::Options => {
                app.page = Page::Options {
                    anime_list: mem::take(anime_list),
                    old_config: app.config.clone(),
                    state: Options::Scraper,
                };
                app.main_menu_selection = MainMenuSelection::Options;
            }
            MainMenuSelection::Exit => exit(0),
        },
        _ => panic!("main menu event handler called for non main menu event"),
    }

    // if let Event::Key(KeyEvent {
    //     code,
    //     kind: KeyEventKind::Press,
    //     ..
    // }) = event::read().expect("Couldn't read event from main menu")
    // {
    //     match code {
    //         KeyCode::Up | KeyCode::Char('k') => app.main_menu_selection.select_previous(),
    //         KeyCode::Down | KeyCode::Char('j') => app.main_menu_selection.select_next(),
    //         KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
    //             if let Some(i) = app.main_menu_selection.selected() {
    //                 let option = MainMenuSelection::iter().nth(i).unwrap();
    //                 match option {
    //                     MainMenuSelection::Search => {
    //                         let MenuState::MainMenu {
    //                             anime_list,
    //                             should_draw_popup,
    //                         } = &mut app.menu_state
    //                         else {
    //                             panic!("Invalid app state in main menu")
    //                         };
    //
    //                         let anime_list = mem::take(anime_list);
    //
    //                         let anime_list = match anime_list {
    //                             ListQueryState::Obtaining(..) => {
    //                                 *should_draw_popup = true;
    //                                 app.draw().unwrap();
    //                                 anime_list.get()
    //                             }
    //                             ListQueryState::Obtained(..) => anime_list,
    //                             _ => panic!("Invalid anime_list state"),
    //                         };
    //
    //                         let ListQueryState::Obtained(anime_list) = anime_list else {
    //                             panic!("Should not happen")
    //                         };
    //
    //                         let filtered_list = anime_list.clone();
    //
    //                         app.menu_state = MenuState::Search {
    //                             anime_list,
    //                             search_state: SearchState::Searching,
    //                             query: String::new(),
    //                             anime_state: ListState::default().with_selected(Some(0)),
    //                             filtered_list,
    //                         }
    //                     }
    //                     MainMenuSelection::Options => {
    //                         let MenuState::MainMenu { anime_list, .. } = &mut app.menu_state else {
    //                             panic!("Invalid app state in main menu")
    //                         };
    //
    //                         app.menu_state = MenuState::Options {
    //                             anime_list: mem::take(anime_list),
    //                             old_config: app.config.clone(),
    //                             state: ListState::default().with_selected(Some(0)),
    //                         }
    //                     }
    //                     MainMenuSelection::Exit => app.running = false,
    //                 }
    //             }
    //         }
    //         KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => app.running = false,
    //         _ => {}
    //     }
    // }
}
