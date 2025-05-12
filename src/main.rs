use animeflvscraper::AnimeFLVScraper;
use clap::Parser;
use dialoguer::{Confirm, FuzzySelect};
use itertools::Itertools;
use reqwest::Client;
use scraper::Scraper;
use std::{error::Error, process::exit};

mod animeflvscraper;
mod scraper;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Query made to the website
    #[arg(short, long, default_value_t = String::new())]
    query: String,
}

async fn run<T: Scraper>() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let client = Client::new();

    let animes = T::try_search(&client, &args.query).await?;

    let display_anime = animes
        .iter()
        .map(|anime| anime.replace("-", " "))
        .collect_vec();

    if animes.is_empty() {
        eprintln!("No se ha encontrado ningún anime que contenga la query especificada");
        exit(1);
    }

    print!("{esc}c", esc = 27 as char);

    let anime_index = FuzzySelect::new()
        .with_prompt("Elige un anime")
        .items(&display_anime)
        .interact()?;
    let anime = &animes[anime_index];

    let episodes = T::try_get_episodes(&client, anime).await?;

    print!("{esc}c", esc = 27 as char);

    let episode_index = FuzzySelect::new()
        .with_prompt("Elige un episodio")
        .items(&episodes)
        .interact()?;
    let episodes = episodes.iter().skip(episode_index);

    for episode in episodes {
        print!("{esc}c", esc = 27 as char);
        println!("Anime: {}:", anime);
        println!("Episodio {}:\n", episode);
        let mirrors = T::try_get_mirrors(&client, anime, *episode).await?;
        println!("{:#?}\n", mirrors);

        let next = Confirm::new()
            .with_prompt("Siguiente episodio?")
            .interact()?;

        if !next {
            return Ok(());
        };
    }

    print!("\x1B[2J\x1B[1;1H");

    println!("No hay más episodios");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run::<AnimeFLVScraper>().await
}
