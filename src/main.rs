use clap::Parser;
use dialoguer::{Confirm, FuzzySelect};
use futures::future;
use itertools::Itertools;
use regex::Regex;
use std::error::Error;

trait Scraper {
    async fn try_search(query: &str, pages: usize) -> Result<Vec<String>, Box<dyn Error>>;
    async fn try_get_episodes(anime: &str) -> Result<Vec<usize>, Box<dyn Error>>;
    async fn try_get_mirrors(anime: &str, episode: usize) -> Result<Vec<String>, Box<dyn Error>>;
}

struct AnimeFLVScraper;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Query made to the website
    #[arg(short, long, default_value_t = String::new())]
    query: String,

    /// Number of pages searched
    #[arg(short, long, default_value_t = 10)]
    pages: usize,
}

impl Scraper for AnimeFLVScraper {
    async fn try_search(query: &str, pages: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let pattern = Regex::new("\"/anime/.*?\"")?;
        let client = reqwest::Client::new();

        let urls = (1..=pages)
            .map(|page| format!("https://www3.animeflv.net/browse?q={}&page={}", query, page))
            .collect_vec();

        let bodies = future::join_all(urls.into_iter().map(|url| {
            let client = &client;
            async move {
                let response = client.get(&url).send().await?;
                response.text().await
            }
        }))
        .await
        .into_iter()
        .filter_map(|request| request.ok())
        .join("\n\n");

        let animes = pattern
            .find_iter(&bodies)
            .map(|found| found.as_str()[8..found.as_str().len() - 1].into())
            .sorted()
            .dedup()
            .collect_vec();

        Ok(animes)
    }

    async fn try_get_episodes(anime: &str) -> Result<Vec<usize>, Box<dyn Error>> {
        let pattern = Regex::new("var episodes = .*?;")?;
        let anime = reqwest::get(format!("https://www3.animeflv.net/anime/{}", anime))
            .await?
            .text()
            .await?;

        let binding = pattern
            .find(&anime)
            .ok_or("episodes variable not found")?
            .as_str()
            .split_whitespace()
            .skip(3)
            .join("");

        let episodes = binding
            .strip_prefix("[[")
            .unwrap()
            .strip_suffix("]];")
            .unwrap()
            .split("],[")
            .filter_map(|ep| ep.split(",").next())
            .filter_map(|ep| ep.parse::<usize>().ok())
            .sorted()
            .collect_vec();

        Ok(episodes)
    }

    async fn try_get_mirrors(anime: &str, episode: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let response = reqwest::get(format!(
            "https://www3.animeflv.net/ver/{}-{}",
            anime, episode
        ))
        .await?
        .text()
        .await?;

        let pattern = Regex::new("\"code\":\".*?\"")?;

        let mirrors = pattern
            .find_iter(&response)
            .map(|found| {
                found
                    .as_str()
                    .to_owned()
                    .replace("\\", "")
                    .replace("\"", "")[5..]
                    .to_owned()
            })
            .collect_vec();

        Ok(mirrors)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let animes = AnimeFLVScraper::try_search(&args.query, args.pages).await?;

    let anime_index = FuzzySelect::new()
        .with_prompt("Elige un anime")
        .items(&animes)
        .interact()?;
    let anime = &animes[anime_index];

    let episodes = AnimeFLVScraper::try_get_episodes(anime).await?;

    let episode_index = FuzzySelect::new()
        .with_prompt("Elige un episodio")
        .items(&episodes)
        .interact()?;
    let episodes = episodes.iter().skip(episode_index);

    for episode in episodes {
        print!("\x1B[2J\x1B[1;1H");
        println!("Anime: {}:", anime);
        println!("Episodio {}:\n", episode);
        let mirrors = AnimeFLVScraper::try_get_mirrors(anime, *episode).await?;
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
