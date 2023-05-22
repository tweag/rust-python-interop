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
            let cats = get_cats().await;

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

async fn get_cats() -> Result<Vec<Cat>, reqwest::Error> {
    let cats: Vec<Cat> = reqwest::get("xhttp://foo").await?.json().await?;

    Ok(cats)
}
