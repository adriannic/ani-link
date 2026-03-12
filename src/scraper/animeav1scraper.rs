use iced::futures::{StreamExt, stream};
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use reqwest::Client;
use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use super::{Scraper, anime::Anime};

const LETTERS: &str = "0ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const PAGES: i32 = 15;

pub struct AnimeAv1Scraper;

impl Scraper for AnimeAv1Scraper {
    async fn try_search(
        client: &Client,
        progress: Arc<AtomicUsize>,
    ) -> Result<Vec<Anime>, Box<dyn Error>> {
        let urls = (1..=PAGES)
            .cartesian_product(LETTERS.chars())
            .par_bridge()
            .map(|(page, letter)| {
                format!("https://animeav1.com/catalogo?letter={letter}&page={page}")
            })
            .collect::<Vec<_>>();

        let total = urls.len();

        let bodies = stream::iter(urls)
            .map(|url| {
                let client = client.clone();
                let completed = progress.clone();
                tokio::spawn(async move {
                    let body = client.get(url).send().await.unwrap().text().await.unwrap();
                    completed.fetch_add(1, Ordering::SeqCst);
                    body
                })
            })
            .buffer_unordered(total)
            .map(|result| result.unwrap())
            .collect::<String>()
            .await;

        let anime_list_re = Regex::new(r"results:\s*\[(.*?)\],").unwrap();
        let anime_list_section = anime_list_re
            .captures_iter(&bodies)
            .par_bridge()
            .filter_map(|c| Some(c.get(0)?.as_str()))
            .collect::<String>()
            .replace('\\', "");

        let anime_re =
            Regex::new(r#"\{id:"(.*?)",title:"(.*?)",synopsis:"(.*?)",categoryId.*?slug:"(.*?)",category.*?\}"#)
                .unwrap();

        let mut animes = anime_re
            .captures_iter(&anime_list_section)
            .par_bridge()
            .filter_map(|c| {
                let id = c.get(1)?.as_str();
                let title = c.get(2)?.as_str();
                let synopsis = c.get(3)?.as_str().replace(r"\n", "\n").replace(r".nn", ". ");
                let slug = c.get(4)?.as_str();
                Some(Anime {
                    names: vec![title.into(), slug.into()],
                    synopsis,
                    image_url: format!("https://cdn.animeav1.com/covers/{id}.jpg"),
                    image_filename: format!("{id}.jpg"),
                })
            })
            .collect::<Vec<_>>();

        animes.par_sort();

        Ok(animes)
    }

    async fn try_get_episodes(client: &Client, slug: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let anime = client
            .get(format!("https://animeav1.com/media/{slug}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let episodes_re = Regex::new(r"number:(.*?)}")?;
        let mut episodes = episodes_re
            .captures_iter(&anime)
            .par_bridge()
            .filter_map(|c| c.get(1))
            .filter_map(|m| m.as_str().parse::<f64>().ok())
            .collect::<Vec<_>>();

        episodes.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Ok(episodes)
    }

    async fn try_get_mirrors(
        client: &Client,
        slug: &str,
        episode: f64,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let response = client
            .get(format!("https://animeav1.com/media/{slug}/{episode}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

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
            .par_bridge()
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .collect::<Vec<_>>();

        Ok(mirrors)
    }

    fn pages() -> usize {
        (1..=PAGES).cartesian_product(LETTERS.chars()).count()
    }
}
