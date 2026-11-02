use derive_more::{From, Into};
use ergo_lib::chain::parameters::Parameters as ParametersInner;
use pyo3::{prelude::*, types::PyType};
use serde::Deserialize;

use crate::from_json;

#[pyclass(eq, from_py_object)]
#[derive(Clone, PartialEq, Eq, From, Into, Deserialize)]
pub(crate) struct Parameters(ParametersInner);

#[pymethods]
impl Parameters {
    #[classmethod]
    fn default(_: &Bound<'_, PyType>) -> Self {
        Self(Default::default())
    }
    #[classmethod]
    fn from_json(_: &Bound<'_, PyType>, json: Bound<'_, PyAny>) -> PyResult<Self> {
        from_json(json)
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}
