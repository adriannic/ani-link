use crate::config::Config;
use crate::gui::main_menu::MainMenuSelection;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListDirection, ListState};
use ratatui::Terminal;
use std::error::Error;
use std::fmt;
use std::io::Stdout;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter)]
enum Options {
    Scraper,
    Pages,
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Scraper => "Scraper",
                Self::Pages => "Páginas",
            }
        )
    }
}

#[allow(clippy::too_many_lines)]
pub async fn options(
    config: &mut Config,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let mut main_state = ListState::default();
    main_state.select(Some(1));

    let mut option_state = ListState::default();
    option_state.select_first();

    let old_config = config.clone();

    loop {
        terminal.draw(|frame| {
            frame.render_widget(Clear, frame.area());

            // Divide areas
            let horizontal = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]);

            let [main_menu_area, right_area] = horizontal.areas(frame.area());

            // Render right block
            let right_title = Line::from(" Opciones ".bold().white()).centered();

            let right_instructions = Line::from(vec![
                " Subir:".white(),
                " ↑ K ".blue().bold(),
                " Bajar:".white(),
                " ↓ J ".blue().bold(),
                " Siguiente:".white(),
                " → L ".blue().bold(),
                " Anterior:".white(),
                " ← H ".blue().bold(),
                " Guardar:".white(),
                " Enter ".blue().bold(),
                " Salir sin guardar:".white(),
                " Esc Q ".blue().bold(),
            ]);

            let right_block = Block::bordered()
                .title(right_title)
                .title_bottom(right_instructions.centered())
                .border_set(border::THICK)
                .border_style(Style::new().green());

            let option_list = List::new(Options::iter().map(|option| {
                format!(
                    "{}: {}",
                    option,
                    match option {
                        Options::Scraper => config.scraper.to_string(),
                        Options::Pages => config.pages.to_string(),
                    }
                )
            }))
            .block(right_block)
            .highlight_symbol("> ")
            .highlight_style(Style::new().bold())
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(option_list, right_area, &mut option_state);

            // Render Main menu option list
            let list_title = Line::from(" Menú principal ".bold().white()).centered();

            let list_block = Block::bordered()
                .title(list_title)
                .border_set(border::THICK)
                .border_style(Style::new().green());

            let items = MainMenuSelection::iter().map(|scraper| scraper.to_string());

            let main_menu_list = List::new(items)
                .block(list_block)
                .highlight_symbol("> ")
                .highlight_style(Style::new().bold())
                .style(Style::new().gray())
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(main_menu_list, main_menu_area, &mut main_state);
        })?;

        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Char('k') | KeyCode::Up => option_state.select_previous(),
                KeyCode::Char('j') | KeyCode::Down => option_state.select_next(),
                KeyCode::Char('l') | KeyCode::Right => {
                    if let Some(i) = option_state.selected() {
                        let option = Options::iter().nth(i).unwrap();
                        match option {
                            Options::Scraper => config.scraper = config.scraper.next(),
                            Options::Pages => config.pages = (config.pages + 1).clamp(1, 50),
                        }
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if let Some(i) = option_state.selected() {
                        let option = Options::iter().nth(i).unwrap();
                        match option {
                            Options::Scraper => config.scraper = config.scraper.previous(),
                            Options::Pages => config.pages = (config.pages - 1).clamp(1, 50),
                        }
                    }
                }
                KeyCode::Enter => {
                    config.save()?;
                    break;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    *config = old_config;
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
