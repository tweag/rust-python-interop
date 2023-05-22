use async_stream::stream;
use pyo3::{exceptions::*, prelude::*, types::*};
use tokio_stream::Stream;

#[derive(serde::Deserialize)]
struct Cat {
    url: String,
}

#[derive(Debug)]
struct IteratorError(reqwest::Error);

pub(crate) struct IteratorResult<T>(Result<T, IteratorError>);

impl From<IteratorError> for PyErr {
    fn from(error: IteratorError) -> Self {
        PyValueError::new_err(error.0.to_string())
    }
}

impl From<reqwest::Error> for IteratorError {
    fn from(other: reqwest::Error) -> Self {
        Self(other)
    }
}

impl IntoPy<Py<PyAny>> for IteratorResult<String> {
    fn into_py(self, py: Python) -> Py<PyAny> {
        match self.0 {
            Ok(val) => PyString::new(py, &val).into(),
            Err(e) => {
                let err = PyErr::new::<PyException, _>(format!("{e:?}"));
                err.into_py(py)
            }
        }
    }
}

impl<T> From<Result<T, IteratorError>> for IteratorResult<T> {
    fn from(value: Result<T, IteratorError>) -> Self {
        IteratorResult(value)
    }
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

pub(crate) fn stream_cats_with_error() -> impl Stream<Item = IteratorResult<String>> {
    stream! {
        loop {
            let cats = get_cats().await;

            match cats {
                Ok(cats) => {
                    for cat in cats {
                        yield IteratorResult(Ok(cat.url));
                    }
                }
                Err(e) => {
                    yield IteratorResult(Err(e.into()));
                }
            }
        }
    }
}

async fn get_cats() -> Result<Vec<Cat>, reqwest::Error> {
    let cats: Vec<Cat> = reqwest::get("xhttp://foo").await?.json().await?;

    Ok(cats)
}
