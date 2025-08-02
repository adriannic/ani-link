use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use reqwest::blocking::Client;
use std::error::Error;

use super::{Scraper, anime::Anime};

pub struct AnimeFlvScraper;

impl Scraper for AnimeFlvScraper {
    fn try_search(client: &Client) -> Result<Vec<Anime>, Box<dyn Error>> {
        let pages = 150;
        let anime_re = Regex::new(
            r#"<article class="Anime alt B">[\s\S]*?"\/anime\/(.*?)"[\s\S]*?(\d+)\.jpg[\s\S]*?Title">(.*?)<[\s\S]*?<\/p>[\s\S]*?<p>([\s\S]*?)<\/p>[\s\S]*?<\/article>"#,
        )?;

        let urls = (1..=pages)
            .into_par_iter()
            .map(|page| format!("https://www3.animeflv.net/browse?page={page}"))
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
                let id = c.get(2)?.as_str();
                let title = c.get(3)?.as_str();
                let synopsis = c.get(4)?.as_str();
                Some(Anime {
                    names: vec![title.into(), slug.into()],
                    synopsis: synopsis.into(),
                    image_url: format!("https://www3.animeflv.net/uploads/animes/covers/{id}.jpg"),
                })
            })
            .collect::<Vec<_>>();

        animes.par_sort();

        Ok(animes)
    }

    fn try_get_episodes(client: &Client, slug: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let pattern = Regex::new("var episodes = .*?;")?;
        let anime = client
            .get(format!("https://www3.animeflv.net/anime/{slug}"))
            .send()
            .unwrap()
            .text()
            .unwrap();

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
            .filter_map(|ep| ep.split(',').next())
            .filter_map(|ep| ep.parse::<f64>().ok())
            .sorted_by(|a, b| a.partial_cmp(b).unwrap())
            .collect_vec();

        Ok(episodes)
    }

    fn try_get_mirrors(
        client: &Client,
        slug: &str,
        episode: f64,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let response = client
            .get(format!("https://www3.animeflv.net/ver/{slug}-{episode}"))
            .send()
            .unwrap()
            .text()
            .unwrap();

        let pattern = Regex::new("\"code\":\".*?\"")?;

        let mirrors = pattern
            .find_iter(&response)
            .map(|found| found.as_str().to_owned().replace(['\\', '"'], "")[5..].to_owned())
            .collect_vec();

        Ok(mirrors)
    }
}
