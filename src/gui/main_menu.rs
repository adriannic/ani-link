use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListDirection, ListState, Paragraph};
use ratatui::Terminal;
use std::error::Error;
use std::fmt;
use std::io::Stdout;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::config::Config;

use super::options::options;
use super::search::search;

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

impl MainMenuSelection {
    pub async fn run(
        self,
        config: &mut Config,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<bool, Box<dyn Error>> {
        match self {
            Self::Search => search(config, terminal).await?,
            Self::Options => options(config, terminal).await?,
            Self::Exit => return Ok(true),
        }

        Ok(false)
    }
}

pub async fn main_menu(
    config: &mut Config,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let mut selected = ListState::default();
    selected.select_first();

    loop {
        draw(terminal, &mut selected, false)?;

        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Up | KeyCode::Char('k') => selected.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => selected.select_next(),
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                    if let Some(i) = selected.selected() {
                        let option = MainMenuSelection::iter().nth(i).unwrap();
                        if option == MainMenuSelection::Search {
                            draw(terminal, &mut selected, true)?;
                        }
                        if option.run(config, terminal).await? {
                            break;
                        }
                    }
                }
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    Ok(())
}

fn get_popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    selected: &mut ListState,
    searching: bool,
) -> Result<(), Box<dyn Error>> {
    terminal.draw(|frame| {
        // Divide areas
        let horizontal = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]);

        let [main_menu_area, right_area] = horizontal.areas(frame.area());

        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Fill(1),
        ]);

        let [_, banner_area, _] = vertical.areas(right_area);

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

        frame.render_widget(right_block, right_area);

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
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(main_menu_list, main_menu_area, selected);

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
    })?;
    Ok(())
}
