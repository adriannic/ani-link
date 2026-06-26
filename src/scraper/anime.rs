use std::fmt;

use bytes::Bytes;
use reqwest::blocking;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Anime {
    pub names: Vec<String>,
    pub synopsis: String,
    pub image_url: String,
}

impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.names[0])
    }
}

impl Anime {
    pub fn get_image(&self) -> Bytes {
        blocking::Client::default()
            .get(&self.image_url)
            .send()
            .expect("Error sending request for image")
            .bytes()
            .expect("Error converting image data to bytes")
    }
}
