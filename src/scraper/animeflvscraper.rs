use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use reqwest::blocking::Client;
use std::error::Error;

use super::{Scraper, anime::Anime};

pub struct AnimeFlvScraper;

impl Scraper for AnimeFlvScraper {
    fn try_search(client: &Client) -> Result<Vec<Anime>, Box<dyn Error>> {
        let pages = 300;
        let anime_re = Regex::new(
            r#"<article class="li">[\s\S]*?<figure class="i">[\s\S]*?<a href=".\/anime\/(.*?)" title="Ver Anime (.*?) Online Gratis">[\s\S]*?data-src="(.*?)\?[\s\S]*?<\/article>"#,
        )?;

        let urls = (1..=pages)
            .into_par_iter()
            .map(|page| format!("https://vww.animeflv.one/animes?pag={page}"))
            .collect::<Vec<_>>();

        let bodies = urls
            .par_iter()
            .map(|url| client.get(url).send().unwrap().text().unwrap())
            .collect::<String>();

        let mut animes = anime_re
            .captures_iter(&bodies)
            .par_bridge()
            .filter_map(|c| {
                let slug = c.get(1)?.as_str();
                let title = c.get(2)?.as_str();
                let img = c.get(3)?.as_str();
                Some(Anime {
                    names: vec![title.into(), slug.into()],
                    synopsis: title.into(),
                    image_url: img.into(),
                })
            })
            .collect::<Vec<_>>();

        animes.par_sort();

        Ok(animes)
    }

    fn try_get_episodes(client: &Client, slug: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let episodes_re = Regex::new(r#"\["(.*?)",[\s\S]*?"0",[\s\S]*?""\]"#)?;
        let anime = client
            .get(format!("https://vww.animeflv.one/anime/{slug}"))
            .send()
            .unwrap()
            .text()
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

    fn try_get_mirrors(
        client: &Client,
        slug: &str,
        episode: f64,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let response = client
            .get(format!("https://vww.animeflv.one/ver/{slug}-{episode}"))
            .send()
            .unwrap()
            .text()
            .unwrap();

        let pattern = Regex::new(r#"(www\.mp4upload.com\\\/.*?)&quot"#)?;

        let mirrors = pattern
            .captures_iter(&response)
            .filter_map(|c| c.get(1))
            .map(|found| {
                let [first, second] = found.as_str().split("\\/").collect_vec()[..2] else {
                    panic!("Shouldn't happen")
                };
                format!("https://{}/embed-{}.html", first, second)
            })
            .collect_vec();

        Ok(mirrors)
    }
}
