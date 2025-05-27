use crate::{scraper::Anime, Scraper};
use async_trait::async_trait;
use futures::future;
use itertools::Itertools;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::error::Error;

pub struct AnimeAV1Scraper;

#[async_trait]
impl Scraper for AnimeAV1Scraper {
    async fn try_search(client: &Client, query: &str) -> Result<Vec<Anime>, Box<dyn Error>> {
        let pages = 100;

        let urls = (1..=pages)
            .map(|page| {
                format!(
                    "https://animeav1.com/catalogo?search={}&page={}",
                    query, page
                )
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
        .filter_map(|request| request.ok())
        .filter_map(|request| request.ok())
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

    async fn try_get_episodes(client: &Client, anime: &str) -> Result<Vec<usize>, Box<dyn Error>> {
        let anime = client
            .get(format!("https://animeav1.com{}", anime))
            .send()
            .await?
            .text()
            .await?;

        let fragment = Html::parse_fragment(&anime);
        let section_selector = Selector::parse("section").unwrap();
        let article_selector = Selector::parse("article").unwrap();

        let episodes = fragment
            .select(&section_selector)
            .next()
            .unwrap()
            .select(&article_selector)
            .count();

        let episodes = (1..=episodes).collect_vec();

        Ok(episodes)
    }

    async fn try_get_mirrors(
        client: &Client,
        anime: &str,
        episode: usize,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let response = client
            .get(format!("https://animeav1.com{}/{}", anime, episode))
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
