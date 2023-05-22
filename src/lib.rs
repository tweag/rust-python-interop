use std::{future::Future, pin::Pin, sync::Arc, task::Poll};

use async_std::{path::Iter, task};
use async_stream::{stream, try_stream};
use pyo3::{
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDateTime, PyString},
};
use serde::Deserialize;
use tokio::{sync::Mutex, time::Duration};
use tokio_stream::{Stream, StreamExt};

#[pyclass]
struct FooStruct {
    #[pyo3(get)]
    msg: String,
    #[pyo3(get)]
    time: Py<PyDateTime>,
}

struct FibonacciIterator {
    curr: usize,
    next: usize,
    sleep: Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>>,
}

impl FibonacciIterator {
    fn new() -> Self {
        Self {
            curr: 0,
            next: 1,
            sleep: Box::pin(task::sleep(Duration::from_millis(100))),
        }
    }
}

impl Iterator for FibonacciIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.curr;
        self.curr = self.next;
        self.next += result;

        Some(result)
    }
}

impl Stream for FibonacciIterator {
    type Item = usize;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match Pin::new(&mut self.sleep).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => {
                self.sleep = Box::pin(task::sleep(Duration::from_millis(50)));

                let result = self.curr;
                self.curr = self.next;
                self.next += result;

                Poll::Ready(Some(result))
            }
        }
    }
}

enum WebIteratorState {
    Init,
    Fetching,
    Finished,
}

struct CatIterator {
    state: WebIteratorState,
}

impl CatIterator {
    fn new() -> Self {
        Self {
            state: WebIteratorState::Init,
        }
    }
}

#[derive(serde::Deserialize)]
struct Cat {
    url: String,
}

fn stream_cats() -> impl Stream<Item = String> {
    stream! {
        loop {
            let cats: Vec<Cat> = reqwest::get("https://api.thecatapi.com/v1/images/search?limit=4").await.unwrap().json().await.unwrap();

            for cat in cats {
                yield cat.url;
            }
        }
    }
}

struct StructIterator {}

impl StructIterator {
    fn new() -> Self {
        Self {}
    }
}

impl Iterator for StructIterator {
    type Item = FooStruct;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| {
            Some(FooStruct {
                msg: "Hello".to_string(),
                time: PyDateTime::new(py, 1999, 12, 12, 1, 1, 1, 1, None)
                    .unwrap()
                    .into(),
            })
        })
    }
}

/// Contains any iterator of usize values.
/// Send is required by Pyo3.
#[pyclass]
struct NumberIteratorSync {
    iter: Box<dyn Iterator<Item = usize> + Send>,
}

#[pymethods]
impl NumberIteratorSync {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<usize> {
        slf.iter.next()
    }
}

/// Contains any iterator of values of struct type FooStruct.
/// Send is required by Pyo3.
#[pyclass]
struct StructIteratorSync {
    iter: Box<dyn Iterator<Item = FooStruct> + Send>,
}

#[pymethods]
impl StructIteratorSync {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<FooStruct> {
        slf.iter.next()
    }
}

/// Box<dyn Stream<Item = usize> + Send> is essentially analogous to the sync example above.
/// Send and Sync is required by Pyo3.
/// Pin is required by the Stream.
/// Mutex is required because we need to mutate the iterator/stream.
/// Arc is required because we need to share the iterator/stream between threads.
#[pyclass]
struct NumberIteratorAsync {
    iter: Arc<Mutex<Pin<Box<dyn Stream<Item = usize> + Send + Sync>>>>,
}

#[pymethods]
impl NumberIteratorAsync {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'a>(slf: PyRefMut<'_, Self>, py: Python<'a>) -> PyResult<Option<&'a PyAny>> {
        let iter = Arc::clone(&slf.iter);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut iter = iter.lock().await;
            Ok(iter.next().await)
        })
        .map(Some)
    }
}

#[derive(Debug)]
struct IteratorError(reqwest::Error);

struct IteratorResult<T>(Result<T, IteratorError>);

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
                let err = PyErr::new::<PyException, _>(format!("{:?}", e));
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

#[pyclass]
struct StringIteratorAsync {
    iter: Arc<Mutex<Pin<Box<dyn Stream<Item = String> + Send + Sync>>>>,
}

#[pymethods]
impl StringIteratorAsync {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'a>(slf: PyRefMut<'_, Self>, py: Python<'a>) -> PyResult<Option<&'a PyAny>> {
        let iter = Arc::clone(&slf.iter);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut iter = iter.lock().await;
            Ok(iter.next().await)
        })
        .map(Some)
    }
}

#[pyfunction]
fn fibonacci_sync() -> NumberIteratorSync {
    NumberIteratorSync {
        iter: Box::new(FibonacciIterator::new()),
    }
}

#[pyfunction]
fn fibonacci_async() -> NumberIteratorAsync {
    NumberIteratorAsync {
        iter: Arc::new(Mutex::new(Box::pin(FibonacciIterator::new()))),
    }
}

#[pyfunction]
fn struct_sync() -> StructIteratorSync {
    StructIteratorSync {
        iter: Box::new(StructIterator::new()),
    }
}

#[pyfunction]
fn cat_async() -> StringIteratorAsync {
    // let pinned_stream: Pin<Box<dyn Stream<Item = IteratorResult<String>> + Send + Sync + 'static>> =
    //     Box::<dyn Stream<Item = IteratorResult<String>> + Send + Sync + 'static>::pin(
    //         stream_cats().into(),
    //     );

    let pinned_stream = Box::pin(stream_cats());

    StringIteratorAsync {
        iter: Arc::new(Mutex::new(pinned_stream)),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn python_async_iterator(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci_sync, m)?)?;
    m.add_function(wrap_pyfunction!(fibonacci_async, m)?)?;
    m.add_function(wrap_pyfunction!(struct_sync, m)?)?;
    m.add_function(wrap_pyfunction!(cat_async, m)?)?;
    m.add_class::<NumberIteratorSync>()?;
    m.add_class::<NumberIteratorAsync>()?;
    m.add_class::<StructIteratorSync>()?;
    m.add_class::<StringIteratorAsync>()?;
    Ok(())
}