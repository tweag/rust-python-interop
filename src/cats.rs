use async_stream::stream;
use tokio_stream::Stream;

#[derive(serde::Deserialize)]
struct Cat {
    url: String,
}

pub(crate) fn stream_cats() -> impl Stream<Item = String> {
    stream! {
        loop {
            let cats: Vec<Cat> = reqwest::get("https://api.thecatapi.com/v1/images/search?limit=4").await.unwrap().json().await.unwrap();

            for cat in cats {
                yield cat.url;
            }
        }
    }
}
