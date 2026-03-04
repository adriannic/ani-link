use dirs::cache_dir;
use iced::advanced::image::Bytes;
use reqwest::blocking::Client;
use std::{
    fs::{create_dir, read, write},
    path::PathBuf,
};

use crate::scraper::ScraperImpl;

#[derive(Debug, Clone)]
pub struct CachedImage {
    client: Client,
    uri: String,
    path: PathBuf,
    bytes: Vec<u8>,
}

impl CachedImage {
    pub fn new(client: Client, uri: String, scraper: ScraperImpl, filename: String) -> Self {
        let mut path = cache_dir().expect("Cache dir not found");

        path.push("ani-link");
        if !path.is_dir() {
            create_dir(&path).expect("Couldn't create app cache dir");
        }

        path.push(scraper.to_string());
        if !path.is_dir() {
            create_dir(&path).expect("Couldn't create scraper cache dir");
        }

        path.push(&filename);

        CachedImage {
            client,
            uri,
            path,
            bytes: vec![],
        }
    }
}

impl From<CachedImage> for Bytes {
    fn from(value: CachedImage) -> Self {
        let mut value = value;
        if !value.bytes.is_empty() {
            return value.bytes.into();
        }

        if value.path.is_file() {
            value.bytes = read(&value.path).expect("Image file not found")
        }

        if !value.bytes.is_empty() {
            return value.bytes.into();
        }

        value.bytes = value
            .client
            .get(&value.uri)
            .send()
            .expect("Error sending request for image")
            .bytes()
            .expect("Error converting image data to bytes")
            .into();

        write(&value.path, &value.bytes).expect("Couldn't create image cache file");

        value.bytes.into()
    }
}
