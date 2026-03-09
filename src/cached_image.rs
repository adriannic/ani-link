use iced::advanced::image::Bytes;
use reqwest::Client;
use tokio::runtime::Handle;

#[derive(Debug, Clone)]
pub struct CachedImage {
    client: Client,
    uri: String,
    bytes: Vec<u8>,
}

impl CachedImage {
    pub fn new(client: Client, uri: String) -> Self {
        CachedImage {
            client,
            uri,
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

        value.bytes = Handle::current()
            .block_on(async {
                value
                    .client
                    .get(&value.uri)
                    .send()
                    .await
                    .expect("Error sending request for image")
                    .bytes()
                    .await
                    .expect("Error converting image data to bytes")
            })
            .into();

        value.bytes.into()
    }
}
