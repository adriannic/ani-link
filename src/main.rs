use animeav1scraper::AnimeAV1Scraper;
use animeflvscraper::AnimeFLVScraper;
use clap::Parser;
use dialoguer::{Confirm, FuzzySelect, Input, Select};
use itertools::Itertools;
use reqwest::Client;
use scraper::{Scraper, ScraperImpl};
use std::{
    error::Error,
    process::{exit, Command},
};
use strum::IntoEnumIterator;

mod animeav1scraper;
mod animeflvscraper;
mod scraper;

const WHITELIST: [&str; 3] = ["mp4upload", "ok.ru", "my.mail.ru"];

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

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
    anime: &scraper::Anime,
) -> Result<Vec<usize>, Box<dyn Error>> {
    let episodes = T::try_get_episodes(client, &anime.url).await?;
    let episode_index = FuzzySelect::new()
        .with_prompt("Elige un episodio")
        .items(&episodes)
        .interact()?;
    let episodes = episodes.iter().skip(episode_index).copied().collect_vec();
    Ok(episodes)
}

async fn select_anime<T: Scraper>(client: &Client) -> Result<scraper::Anime, Box<dyn Error>> {
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

async fn run<T: Scraper>() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let anime = select_anime::<T>(&client).await?;

    let episodes = get_episodes::<T>(&client, &anime).await?;

    for episode in episodes {
        print!("{esc}c", esc = 27 as char);
        println!("Anime: {}", anime.name);
        println!("Episodio {}", episode);
        let mirrors = T::try_get_mirrors(&client, &anime.url, episode).await?;

        let success = try_open::<T>(&mirrors);

        if !success {
            println!(
                "No se ha podido abrir el episodio en mpv, utiliza uno de los siguientes mirrors:"
            );
            println!("{:#?}\n", mirrors);
        }

        let next = Confirm::new()
            .with_prompt("Siguiente episodio?")
            .interact()?;

        if !next {
            return Ok(());
        };
    }

    print!("{esc}c", esc = 27 as char);
    eprint!("{esc}c", esc = 27 as char);

    println!("No hay más episodios");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Args::parse();
    print!("{esc}c", esc = 27 as char);
    eprint!("{esc}c", esc = 27 as char);
    let scrapers = ScraperImpl::iter().collect_vec();

    let index = Select::new()
        .with_prompt("Selecciona el backend")
        .items(&scrapers)
        .interact()?;

    let scraper = scrapers[index];

    match scraper {
        ScraperImpl::AnimeAV1Scraper => run::<AnimeAV1Scraper>().await,
        ScraperImpl::AnimeFLVScraper => run::<AnimeFLVScraper>().await,
    }
}
