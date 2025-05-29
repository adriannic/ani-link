use crate::scraper::Anime;
use crate::scraper::Scraper;
use crate::scraper::ScraperImpl;
use clap::Args;
use dialoguer::FuzzySelect;
use dialoguer::Input;
use itertools::Itertools;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEvent;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::List;
use ratatui::widgets::ListDirection;
use ratatui::widgets::ListState;
use ratatui::widgets::StatefulWidget;
use ratatui::Terminal;
use reqwest::Client;
use std::error::Error;
use std::fmt;
use std::io::Stdout;
use std::process::exit;
use std::process::Command;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const WHITELIST: [&str; 3] = ["mp4upload", "ok.ru", "my.mail.ru"];

#[derive(EnumIter)]
enum MainMenuSelection {
    Search,
    ChangeBackend,
    Exit,
}

impl fmt::Display for MainMenuSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Search => "Buscar",
                Self::ChangeBackend => "Cambiar backend",
                Self::Exit => "Salir",
            }
        )
    }
}

impl MainMenuSelection {
    pub async fn run(
        self,
        app: &mut App,
        client: &Client,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<bool, Box<dyn Error>> {
        match self {
            Self::Search => app.search(client, terminal).await?,
            Self::ChangeBackend => app.change_backend(client, terminal).await?,
            Self::Exit => return Ok(true),
        };
        Ok(false)
    }
}

pub struct App {
    pub scraper: ScraperImpl,
}

impl App {
    pub fn new(_args: impl Args) -> Self {
        App {
            scraper: ScraperImpl::AnimeAV1Scraper,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let mut term = ratatui::init();

        self.main_menu(&client, &mut term).await?;

        ratatui::restore();

        Ok(())
    }

    pub async fn main_menu(
        &mut self,
        client: &Client,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut selected = ListState::default();
        selected.select_first();

        loop {
            terminal.draw(|frame| {
                let title = Line::from(format!("Ani-link v{}", env!("CARGO_PKG_VERSION")).bold())
                    .centered();

                let instructions = Line::from(vec![
                    " Subir ".white(),
                    " <UpArrow | K> ".green().bold(),
                    " Bajar ".white(),
                    " <DownArrow | J> ".green().bold(),
                    " Confirmar ".white(),
                    " <Enter | L> ".green().bold(),
                    " Salir ".white(),
                    " <Esc | Q> ".green().bold(),
                ]);

                let block = Block::bordered()
                    .title(title)
                    .title_bottom(instructions.centered())
                    .border_set(border::THICK)
                    .border_style(Style::new().blue());

                let items = MainMenuSelection::iter().map(|scraper| scraper.to_string());

                List::new(items)
                    .block(block)
                    .highlight_symbol("> ")
                    .highlight_style(Style::new().bold())
                    .repeat_highlight_symbol(true)
                    .direction(ListDirection::TopToBottom)
                    .render(frame.area(), frame.buffer_mut(), &mut selected);
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
                            if option.run(self, client, terminal).await? {
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

    pub async fn search(
        &self,
        client: &Client,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        Command::new("notify-send")
            .args(["Ani-link", "search"])
            .output()
            .unwrap();
        Ok(())
    }

    pub async fn change_backend(
        &mut self,
        client: &Client,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        Command::new("notify-send")
            .args(["Ani-link", "change_backend"])
            .output()
            .unwrap();
        Ok(())
    }
}

fn try_open<T: Scraper>(mirrors: &[String]) -> bool {
    let viewable = mirrors
        .iter()
        .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
        .collect_vec();

    println!("Intentando abrir en mpv...");
    let success = viewable.iter().any(|mirror| {
        println!("Intentando {}...", mirror);

        let mut command = if cfg!(target_os = "windows") {
            Command::new("mpv.exe")
        } else {
            Command::new("mpv")
        };

        command
            .arg(mirror)
            .output()
            .ok()
            .and_then(|output| output.status.code().filter(|&code| code == 0))
            .is_some()
    });
    success
}

async fn get_episodes<T: Scraper>(
    client: &Client,
    anime: &Anime,
) -> Result<Vec<usize>, Box<dyn Error>> {
    let episodes = T::try_get_episodes(client, &anime.url).await?;
    let episode_index = FuzzySelect::new()
        .with_prompt("Elige un episodio")
        .items(&episodes)
        .interact()?;
    let episodes = episodes.iter().skip(episode_index).copied().collect_vec();
    Ok(episodes)
}

async fn select_anime<T: Scraper>(client: &Client) -> Result<Anime, Box<dyn Error>> {
    let query: String = Input::new().with_prompt("Buscar anime").interact()?;
    let animes = T::try_search(client, &query).await?;
    if animes.is_empty() {
        eprintln!("No se ha encontrado ningún anime.");
        exit(1);
    }
    let display_anime = animes.iter().map(|anime| anime.name.as_str()).collect_vec();
    let anime_index = FuzzySelect::new()
        .with_prompt("Elige un anime")
        .items(&display_anime)
        .interact()?;
    let anime = animes[anime_index].clone();
    Ok(anime)
}
