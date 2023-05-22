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

pub(crate) fn stream_cats_with_error() -> impl Stream<Item = Result<String, reqwest::Error>> {
    stream! {
        loop {
            // This will intentionally fail
            let cats: Result<Vec<Cat>, reqwest::Error> = reqwest::get("foo://bar").await?.json().await;

            match cats {
                Ok(cats) => {
                    for cat in cats {
                        yield Ok(cat.url);
                    }
                }
                Err(e) => {
                    yield Err(e);
                }
            }
        }
    }
}
