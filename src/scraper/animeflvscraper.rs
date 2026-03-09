use iced::futures::{StreamExt, stream};
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use reqwest::Client;
use std::sync::atomic::Ordering;
use std::{
    error::Error,
    sync::{Arc, atomic::AtomicUsize},
};

use super::{Scraper, anime::Anime};

pub struct AnimeFlvScraper;

const PAGES: usize = 300;

impl Scraper for AnimeFlvScraper {
    async fn try_search(
        client: &Client,
        progress: Arc<AtomicUsize>,
    ) -> Result<Vec<Anime>, Box<dyn Error>> {
        let anime_re = Regex::new(
            r#"<article class="li">[\s\S]*?<figure class="i">[\s\S]*?<a href=".\/anime\/(.*?)" title="Ver Anime (.*?) Online Gratis">[\s\S]*?data-src="(.*?)\?[\s\S]*?<\/article>"#,
        )?;

        let urls = (1..=PAGES)
            .par_bridge()
            .map(|page| format!("https://vww.animeflv.one/animes?pag={page}"))
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

        let mut animes = anime_re
            .captures_iter(&bodies)
            .par_bridge()
            .filter_map(|c| {
                let slug = c.get(1)?.as_str();
                let id = c.get(2)?.as_str();
                let title = c.get(3)?.as_str().replace(r#"\""#, r#"""#);
                let synopsis = c
                    .get(4)?
                    .as_str()
                    .replace(r#"\""#, r#"""#)
                    .replace(r"\n", "\n");
                Some(Anime {
                    names: vec![title, slug.into()],
                    synopsis,
                    image_url: format!("https://www3.animeflv.net/uploads/animes/covers/{id}.jpg"),
                    image_filename: format!("{id}.jpg"),
                })
            })
            .collect::<Vec<_>>();

        animes.par_sort();

        Ok(animes)
    }

    async fn try_get_episodes(client: &Client, slug: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let episodes_re = Regex::new(r#"\["(.*?)",[\s\S]*?"0",[\s\S]*?""\]"#)?;
        let anime = client
            .get(format!("https://vww.animeflv.one/anime/{slug}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

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
            .get(format!("https://vww.animeflv.one/ver/{slug}-{episode}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let pattern = Regex::new(r"(www\.mp4upload.com\\\/.*?)&quot")?;

        let mirrors = pattern
            .captures_iter(&response)
            .filter_map(|c| c.get(1))
            .map(|found| {
                let [first, second] = found.as_str().split("\\/").collect_vec()[..2] else {
                    panic!("Shouldn't happen")
                };
                format!("https://{first}/embed-{second}.html")
            })
            .collect_vec();

        Ok(mirrors)
    }

    fn pages() -> usize {
        PAGES
    }
}
