use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use reqwest::blocking::Client;
use std::error::Error;

use super::{Scraper, anime::Anime};

const LETTERS: &str = "0ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub struct AnimeAv1Scraper;

impl Scraper for AnimeAv1Scraper {
    fn try_search(client: &Client) -> Result<Vec<Anime>, Box<dyn Error>> {
        let pages = 11;

        let urls = (1..=pages)
            .cartesian_product(LETTERS.chars())
            .par_bridge()
            .map(|(page, letter)| {
                format!("https://animeav1.com/catalogo?letter={letter}&page={page}")
            })
            .collect::<Vec<_>>();

        let bodies = urls
            .par_iter()
            .map(|url| client.get(url).send().unwrap().text().unwrap())
            .collect::<String>();

        let animes_re = Regex::new(r"results:\s*\[(.*?)\],").unwrap();
        let animes_section = animes_re
            .captures_iter(&bodies)
            .par_bridge()
            .filter_map(|c| Some(c.get(0)?.as_str()))
            .collect::<String>();

        let anime_re =
            Regex::new(r#"\{id:"(.*?)",title:"(.*?)",synopsis:"(.*?)",.*?slug:"(.*?)",.*?\}"#)
                .unwrap();

        let mut animes = anime_re
            .captures_iter(&animes_section)
            .par_bridge()
            .filter_map(|c| {
                let id = c.get(1)?.as_str();
                let title = c.get(2)?.as_str();
                let synopsis = c.get(3)?.as_str();
                let slug = c.get(4)?.as_str();
                Some(Anime {
                    names: vec![title.into(), slug.into()],
                    synopsis: synopsis.into(),
                    image_url: format!("https://cdn.animeav1.com/covers/{id}.jpg"),
                })
            })
            .collect::<Vec<_>>();

        animes.par_sort();

        Ok(animes)
    }

    fn try_get_episodes(client: &Client, slug: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let anime = client
            .get(format!("https://animeav1.com/media/{slug}"))
            .send()
            .unwrap()
            .text()
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

    fn try_get_mirrors(
        client: &Client,
        slug: &str,
        episode: f64,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let response = client
            .get(format!("https://animeav1.com/media/{slug}/{episode}"))
            .send()
            .unwrap()
            .text()
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
}

#[cfg(test)]
mod test {
    use reqwest::blocking::Client;

    use crate::scraper::Scraper;

    use super::AnimeAv1Scraper;

    #[test]
    fn av1_get_animes() {
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            )
            .cookie_store(true)
            .build()
            .expect("Couldn't build client");

        let animes = AnimeAv1Scraper::try_search(&client).expect("Animes not retrieved");
        println!("{animes:#?}");
        println!("length: {}", animes.len());
    }

    #[test]
    fn av1_get_episodes() {
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            )
            .cookie_store(true)
            .build()
            .expect("Couldn't build client");

        let anime = "no-game-no-life";
        let episodes =
            AnimeAv1Scraper::try_get_episodes(&client, anime).expect("Episodes not found");
        println!("anime: {anime}");
        println!("{episodes:#?}");
    }

    #[test]
    fn av1_get_mirrors() {
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            )
            .cookie_store(true)
            .build()
            .expect("Couldn't build client");

        let anime = "no-game-no-life";
        let episodes =
            AnimeAv1Scraper::try_get_episodes(&client, anime).expect("Episodes not found");
        let mirrors = AnimeAv1Scraper::try_get_mirrors(&client, anime, episodes[0])
            .expect("Mirrors not found");
        println!("{mirrors:#?}");
        println!("episode: {}", episodes[0]);
    }
}
