use futures::future;
use itertools::Itertools;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::error::Error;

use super::{anime::Anime, Scraper};

const LETTERS: &str = "0ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub struct AnimeAv1Scraper;

impl Scraper for AnimeAv1Scraper {
    async fn try_search(client: &Client) -> Result<Vec<Anime>, Box<dyn Error>> {
        let pages = 11;
        let urls = (1..=pages)
            .cartesian_product(LETTERS.chars())
            .map(|(page, letter)| {
                format!("https://animeav1.com/catalogo?letter={letter}&page={page}")
            })
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
        .filter_map(Result::ok)
        .filter_map(Result::ok)
        .join("\n\n");

        let fragment = Html::parse_fragment(&bodies);
        let article_selector = Selector::parse("article").unwrap();
        let header_selector = Selector::parse("header").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let animes = fragment
            .select(&article_selector)
            .filter_map(|article| {
                let header = article.select(&header_selector).next()?;
                let a = article.select(&a_selector).next()?;

                let name = header.text().next()?.to_string();
                let url = a.attr("href")?.to_owned();

                Some(Anime { url, name })
            })
            .collect_vec();

        Ok(animes)
    }

    async fn try_get_episodes(client: &Client, anime: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let anime = client
            .get(format!("https://animeav1.com{anime}"))
            .send()
            .await?
            .text()
            .await?;

        let fragment = Html::parse_fragment(&anime);
        let article_selector = Selector::parse("article").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let episodes = fragment
            .select(&article_selector)
            .map(|article| {
                article
                    .select(&a_selector)
                    .next()
                    .unwrap()
                    .attr("href")
                    .unwrap()
                    .split('/')
                    .next_back()
                    .unwrap()
                    .to_owned()
            })
            .filter_map(|elem| elem.parse::<f64>().ok())
            .collect_vec();

        Ok(episodes)
    }

    async fn try_get_mirrors(
        client: &Client,
        anime: &str,
        episode: f64,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let response = client
            .get(format!("https://animeav1.com{anime}/{episode}"))
            .send()
            .await?
            .text()
            .await?;

        let embeds_re = Regex::new(r"embeds:\s*\{([^}]*\{[^}]*\})*[^}]*\}")?;
        let embeds_section = embeds_re
            .captures(&response)
            .and_then(|c| c.get(0))
            .ok_or("Embeds section not found")?
            .as_str();

        let sub_re = Regex::new(r"SUB:\s*\[([^]]*\{[^]]*\})*[^]]*\]")?;
        let sub_section = sub_re
            .captures(embeds_section)
            .and_then(|c| c.get(0))
            .ok_or("Sub section not found")?
            .as_str();

        // Then find all URLs within the embeds section
        let url_re = Regex::new(r#"url:\s*"([^"]+)"#)?;
        let mirrors: Vec<String> = url_re
            .captures_iter(sub_section)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .collect();

        Ok(mirrors)
    }
}
