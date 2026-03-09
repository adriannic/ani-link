use std::sync::{Arc, atomic::AtomicUsize};

use reqwest::Client;
use tokio::{runtime::Handle, task::JoinHandle};

use crate::scraper::{ScraperImpl, anime::Anime};

pub enum ListQueryState {
    Obtaining(JoinHandle<Vec<Anime>>, Arc<AtomicUsize>),
    Obtained(Vec<Anime>, Arc<AtomicUsize>),
}

impl Default for ListQueryState {
    fn default() -> Self {
        Self::Obtained(vec![], Arc::new(AtomicUsize::new(0)))
    }
}

impl ListQueryState {
    pub fn spawn(scraper: ScraperImpl, client: Client) -> Self {
        let progress = Arc::new(AtomicUsize::new(0));
        let progress2 = progress.clone();
        ListQueryState::Obtaining(
            tokio::spawn(async move {
                scraper
                    .try_search(&client, progress)
                    .await
                    .expect("Couldn't retrieve the list of animes")
            }),
            progress2,
        )
    }

    pub fn get(self) -> Self {
        match self {
            Self::Obtaining(handle, progress) => Self::Obtained(
                Handle::current()
                    .block_on(handle)
                    .expect("Thread couldn't be joined"),
                progress,
            ),
            Self::Obtained(..) => self,
        }
    }
}
