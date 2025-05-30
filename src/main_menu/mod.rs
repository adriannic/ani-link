use options::options;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListDirection, ListState, StatefulWidget};
use ratatui::Terminal;
use reqwest::Client;
use search::search;
use std::error::Error;
use std::fmt;
use std::io::Stdout;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::config::Config;

mod options;
mod search;

#[derive(EnumIter)]
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

impl MainMenuSelection {
    pub async fn run(
        self,
        config: &Config,
        client: &Client,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<bool, Box<dyn Error>> {
        match self {
            Self::Search => search(config, client, terminal).await?,
            Self::Options => options(config, client, terminal).await?,
            Self::Exit => return Ok(true),
        };
        Ok(false)
    }
}

pub async fn main_menu(
    config: &Config,
    client: &Client,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let mut selected = ListState::default();
    selected.select_first();

    loop {
        terminal.draw(|frame| {
            let vertical = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]);

            let [main_menu_area, right_area] = vertical.areas(frame.area());

            let right_title =
                Line::from(format!("Ani-link v{}", env!("CARGO_PKG_VERSION")).bold()).centered();

            let right_instructions = Line::from(vec![
                " Subir ".white(),
                " <UpArrow | K> ".green().bold(),
                " Bajar ".white(),
                " <DownArrow | J> ".green().bold(),
                " Confirmar ".white(),
                " <Enter | L> ".green().bold(),
                " Salir ".white(),
                " <Esc | Q> ".green().bold(),
            ]);

            let right_block = Block::bordered()
                .title(right_title)
                .title_bottom(right_instructions.centered())
                .border_set(border::THICK)
                .border_style(Style::new().blue());

            frame.render_widget(right_block, right_area);

            let list_block = Block::bordered()
                .border_set(border::THICK)
                .border_style(Style::new().blue());

            let items = MainMenuSelection::iter().map(|scraper| scraper.to_string());

            let main_menu_list = List::new(items)
                .block(list_block)
                .highlight_symbol("> ")
                .highlight_style(Style::new().bold())
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(main_menu_list, main_menu_area, &mut selected);
        })?;

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('j') | KeyCode::Down => {
                    selected.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    selected.select_previous();
                }
                KeyCode::Char('l') | KeyCode::Enter => {
                    if let Some(i) = selected.selected() {
                        let option = MainMenuSelection::iter().nth(i).unwrap();
                        if option.run(config, client, terminal).await? {
                            break;
                        }
                    }
                }
                _ => {}
            };
        }
    }
    Ok(())
}
