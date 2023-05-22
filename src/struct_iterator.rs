use pyo3::{prelude::*, types::*};

#[pyclass]
pub(crate) struct FooStruct {
    #[pyo3(get)]
    msg: String,
    #[pyo3(get)]
    time: Py<PyDateTime>,
}

pub(crate) struct StructIterator {}

impl StructIterator {
    pub(crate) fn new() -> Self {
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
