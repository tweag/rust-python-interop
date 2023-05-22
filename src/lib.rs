use std::{collections::HashMap, pin::Pin, sync::Arc};

use cats::*;
use fibonacci::*;
use pyo3::{prelude::*, types::PyDateTime};
use struct_iterator::*;
use tokio::sync::Mutex;
use tokio_stream::{Stream, StreamExt};

mod cats;
mod fibonacci;
mod struct_iterator;

#[pyclass]
struct SomeData {
    num: usize,
    msg: String,
    timestamp: f64,
    dict: HashMap<bool, usize>,
}

#[pyfunction]
fn get_data() -> SomeData {
    SomeData {
        num: 2,
        msg: "Hello".to_string(),
        timestamp: 1524885322.0,
        dict: HashMap::from([(false, 2), (true, 4)]),
    }
}

#[pymethods]
impl SomeData {
    #[getter]
    fn num(&self) -> usize {
        self.num
    }

    #[getter]
    fn msg(&self) -> &str {
        &self.msg
    }

    #[getter]
    fn date<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyDateTime> {
        PyDateTime::from_timestamp(py, self.timestamp, None)
    }

    #[getter]
    fn dict(&self) -> HashMap<bool, usize> {
        self.dict.clone()
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

#[pyclass]
struct StringResultIteratorAsync {
    iter: Arc<Mutex<Pin<Box<dyn Stream<Item = Result<String, reqwest::Error>> + Send + Sync>>>>,
}

#[pymethods]
impl StringResultIteratorAsync {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'a>(slf: PyRefMut<'_, Self>, py: Python<'a>) -> PyResult<Option<&'a PyAny>> {
        let iter = Arc::clone(&slf.iter);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut iter = iter.lock().await;
            let next = iter.next().await;

            match next {
                Some(Ok(s)) => Ok(s),
                Some(Err(e)) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Error: {}",
                    e
                ))),
                None => Err(PyErr::new::<pyo3::exceptions::PyStopIteration, _>("")),
            }
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
fn cats_async() -> StringIteratorAsync {
    StringIteratorAsync {
        iter: Arc::new(Mutex::new(Box::pin(stream_cats()))),
    }
}

#[pyfunction]
fn cats_with_error_async() -> StringResultIteratorAsync {
    StringResultIteratorAsync {
        iter: Arc::new(Mutex::new(Box::pin(stream_cats_with_error()))),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn python_async_iterator(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_data, m)?)?;
    m.add_function(wrap_pyfunction!(fibonacci_sync, m)?)?;
    m.add_function(wrap_pyfunction!(fibonacci_async, m)?)?;
    m.add_function(wrap_pyfunction!(struct_sync, m)?)?;
    m.add_function(wrap_pyfunction!(cats_async, m)?)?;
    m.add_function(wrap_pyfunction!(cats_with_error_async, m)?)?;
    m.add_class::<NumberIteratorSync>()?;
    m.add_class::<NumberIteratorAsync>()?;
    m.add_class::<StructIteratorSync>()?;
    m.add_class::<StringIteratorAsync>()?;
    Ok(())
}
