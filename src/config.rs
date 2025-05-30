use std::error::Error;
use std::fs::File;
use std::io::Write;

use clap::Parser;
use dirs::config_dir;
use figment::providers::Env;
use figment::providers::Format;
use figment::providers::Serialized;
use figment::providers::Toml;
use figment::Figment;
use serde::Deserialize;
use serde::Serialize;

use crate::scraper::ScraperImpl;

#[derive(Serialize, Deserialize, Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    #[clap(short, long, value_parser, default_value_t = ScraperImpl::AnimeAv1Scraper)]
    pub scraper: ScraperImpl,
    #[clap(short, long, value_parser, default_value_t = 5)]
    pub pages: usize,
}

impl Config {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let mut config_path = config_dir().expect("Config path not found");
        config_path.push("ani-link.toml");

        let config: Config = Figment::new()
            .merge(Serialized::defaults(Config::parse()))
            .merge(Toml::file(config_path.clone()))
            .merge(Env::prefixed("ANI_LINK_"))
            .extract()?;

        let conf_str = toml::to_string(&config)?;
        File::create_new(config_path)
            .and_then(|mut file| file.write_all(conf_str.as_bytes()))
            .ok();

        Ok(config)
    }
}
