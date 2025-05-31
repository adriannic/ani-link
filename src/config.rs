use std::error::Error;
use std::fs::File;
use std::io::Write;

use dirs::config_dir;
use figment::providers::Format;
use figment::providers::Toml;
use figment::Figment;
use serde::Deserialize;
use serde::Serialize;

use crate::scraper::ScraperImpl;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub scraper: ScraperImpl,
    pub pages: usize,
}

impl Config {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let mut config_path = config_dir().expect("Config path not found");
        config_path.push("ani-link.toml");

        let config: Config = Figment::new()
            .merge(Toml::file(config_path.clone()))
            .extract()?;

        config.save()?;

        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut config_path = config_dir().expect("Config path not found");
        config_path.push("ani-link.toml");

        let conf_str = toml::to_string(&self)?;
        File::create(config_path)
            .and_then(|mut file| file.write_all(conf_str.as_bytes()))
            .ok();

        Ok(())
    }
}
