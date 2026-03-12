use iced::advanced::image::Bytes;
use std::{
    env::temp_dir,
    fmt,
    fs::{create_dir_all, read, write},
};

use reqwest::blocking;

use crate::scraper::ScraperImpl;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Anime {
    pub names: Vec<String>,
    pub synopsis: String,
    pub image_url: String,
    pub image_filename: String,
}

impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.names[0])
    }
}

impl Anime {
    pub fn get_image(&self, scraper: ScraperImpl) -> Bytes {
        let mut path = temp_dir();
        path.push("ani-link");
        path.push(scraper.to_string());

        if !path.is_dir() {
            create_dir_all(&path).expect("Couldn't create image dir");
        }

        path.push(
            self.image_url
                .split('/')
                .next_back()
                .expect("Error parsing image url"),
        );

        if path.is_file() {
            return read(path).expect("Couldn't read image file").into();
        }

        let image = blocking::Client::default()
            .get(&self.image_url)
            .send()
            .expect("Error sending request for image")
            .bytes()
            .expect("Error converting image data to bytes");

        write(path, &image).expect("Couldn't save image");

        image
    }
}
