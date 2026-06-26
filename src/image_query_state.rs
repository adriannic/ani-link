use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bytes::Bytes;

use iced::widget::image::Handle;
use reqwest::Client;
use tokio::task::JoinHandle;
pub enum ImageQueryState {
    Obtaining(JoinHandle<Bytes>, Arc<AtomicBool>),
    Obtained(Handle),
}

impl Default for ImageQueryState {
    fn default() -> Self {
        Self::Obtained(Handle::from_bytes(vec![]))
    }
}

impl ImageQueryState {
    pub fn spawn(client: Client, image_url: String) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let done2 = done.clone();

        Self::Obtaining(
            tokio::spawn(async move {
                let done = done2;
                let image = client
                    .get(image_url)
                    .send()
                    .await
                    .expect("Error sending request for image")
                    .bytes()
                    .await
                    .expect("Error converting image data to bytes");

                done.store(true, Ordering::Relaxed);

                image
            }),
            done,
        )
    }

    pub fn get(self) -> Self {
        match self {
            Self::Obtaining(handle, done) => {
                if done.load(Ordering::Relaxed) {
                    Self::Obtained(Handle::from_bytes(
                        tokio::runtime::Handle::current()
                            .block_on(handle)
                            .expect("Thread couldn't be joined"),
                    ))
                } else {
                    Self::Obtaining(handle, done)
                }
            }
            Self::Obtained(..) => self,
        }
    }
}
