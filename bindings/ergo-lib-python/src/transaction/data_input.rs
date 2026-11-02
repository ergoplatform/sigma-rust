use derive_more::{From, Into};
use ergo_lib::chain::transaction::DataInput as DataInputInner;
use pyo3::prelude::*;

use crate::chain::ergo_box::BoxId;

#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Copy, Clone, From, Into)]
pub(crate) struct DataInput(DataInputInner);

#[pymethods]
impl DataInput {
    #[new]
    fn new(box_id: BoxId) -> Self {
        Self(DataInputInner {
            box_id: box_id.into(),
        })
    }
    #[getter]
    fn box_id(&self) -> BoxId {
        self.0.box_id.into()
    }
    fn __repr__(&self) -> String {
        format!("DataInput({:?})", self.0.box_id)
    }
}
