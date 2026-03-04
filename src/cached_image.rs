use dirs::cache_dir;
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
    scraper: ScraperImpl,
    filename: String,
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
            scraper,
            filename,
            path,
            bytes: vec![],
        }
    }

    pub fn to_bytes(mut self) -> Vec<u8> {
        if !self.bytes.is_empty() {
            return self.bytes;
        }

        if self.path.is_file() {
            self.bytes = read(&self.path).expect("Image file not found")
        }

        if !self.bytes.is_empty() {
            return self.bytes;
        }

        self.bytes = self
            .client
            .get(&self.uri)
            .send()
            .expect("Error sending request for image")
            .bytes()
            .expect("Error converting image data to bytes")
            .into();

        write(&self.path, &self.bytes).expect("Couldn't create image cache file");

        self.bytes
    }
}
