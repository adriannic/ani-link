use crate::{scraper::Anime, Scraper};
use async_trait::async_trait;
use futures::future;
use itertools::Itertools;
use regex::Regex;
use reqwest::Client;
use std::error::Error;

pub struct AnimeFLVScraper;

#[async_trait]
impl Scraper for AnimeFLVScraper {
    async fn try_search(client: &Client, query: &str) -> Result<Vec<Anime>, Box<dyn Error>> {
        let pattern = Regex::new("\"/anime/.*?\"")?;
        let pages = 200;

        let urls = (1..=pages)
            .map(|page| format!("https://www3.animeflv.net/browse?q={}&page={}", query, page))
            .collect_vec();

        let bodies = future::join_all(urls.into_iter().map(|url| {
            let client = client.clone();
            tokio::spawn(async move {
                let response = client.get(&url).send().await?;
                response.text().await
            })
        }))
        .await
        .into_iter()
        .filter_map(|request| request.ok())
        .filter_map(|request| request.ok())
        .join("\n\n");

        let animes = pattern
            .find_iter(&bodies)
            .map(|found| {
                let url: String = found.as_str()[8..found.as_str().len() - 1].into();
                let name = url.replace("-", " ");
                Anime {
                    url,
                    name,
                }
            })
            .sorted()
            .dedup()
            .collect_vec();

        Ok(animes)
    }

    async fn try_get_episodes(client: &Client, anime: &str) -> Result<Vec<usize>, Box<dyn Error>> {
        let pattern = Regex::new("var episodes = .*?;")?;
        let anime = client
            .get(format!("https://www3.animeflv.net/anime/{}", anime))
            .send()
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

    async fn try_get_mirrors(
        client: &Client,
        anime: &str,
        episode: usize,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let response = client
            .get(format!(
                "https://www3.animeflv.net/ver/{}-{}",
                anime, episode
            ))
            .send()
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
