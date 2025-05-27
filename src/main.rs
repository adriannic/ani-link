use animeav1scraper::AnimeAV1Scraper;
use animeflvscraper::AnimeFLVScraper;
use clap::Parser;
use dialoguer::{Confirm, FuzzySelect};
use itertools::Itertools;
use reqwest::Client;
use scraper::{Scraper, ScraperImpl};
use std::{
    error::Error,
    process::{exit, Command},
};

mod animeav1scraper;
mod animeflvscraper;
mod scraper;

const WHITELIST: [&str; 4] = ["ok.ru", "mp4upload", "hqq.tv", "my.mail.ru"];

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Query made to the website
    #[arg(default_value_t = String::new())]
    query: String,
    /// Scraper type
    #[arg(short, long, value_enum, default_value_t = ScraperImpl::AnimeAV1Scraper)]
    scraper: ScraperImpl,
}

async fn run<T: Scraper>(args: Args) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let animes = T::try_search(&client, &args.query).await?;

    if animes.is_empty() {
        eprintln!("No se ha encontrado ningún anime.");
        exit(1);
    }

    let display_anime = animes.iter().map(|anime| anime.name.as_str()).collect_vec();

    print!("{esc}c", esc = 27 as char);
    eprint!("{esc}c", esc = 27 as char);

    let anime_index = FuzzySelect::new()
        .with_prompt("Elige un anime")
        .items(&display_anime)
        .interact()?;
    let anime = &animes[anime_index];
    let anime_name = anime.name.as_str();

    let episodes = T::try_get_episodes(&client, &anime.url).await?;

    print!("{esc}c", esc = 27 as char);
    eprint!("{esc}c", esc = 27 as char);

    println!("Anime: {}:", anime_name);
    let episode_index = FuzzySelect::new()
        .with_prompt("Elige un episodio")
        .items(&episodes)
        .interact()?;
    let episodes = episodes.iter().skip(episode_index);

    for episode in episodes {
        print!("{esc}c", esc = 27 as char);
        println!("Anime: {}:", anime_name);
        println!("Episodio {}:", episode);
        let mirrors = T::try_get_mirrors(&client, &anime.url, *episode).await?;

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
    let args = Args::parse();

    match args.scraper {
        ScraperImpl::AnimeFLVScraper => run::<AnimeFLVScraper>(args).await,
        ScraperImpl::AnimeAV1Scraper => run::<AnimeAV1Scraper>(args).await,
    }
}
