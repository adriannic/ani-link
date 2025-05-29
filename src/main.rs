use app::App;
use clap::Parser;
use std::error::Error;

mod animeav1scraper;
mod animeflvscraper;
mod app;
mod scraper;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    App::new(args).run().await
}
