use derive_more::{AsRef, From, Into};
use ergo_lib::chain::ergo_state_context::{
    ErgoStateContext as ErgoStateContextInner, Headers as HeadersInner,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::{
    header::{Header, PreHeader},
    parameters::Parameters,
};

#[pyclass(eq, from_py_object)]
#[derive(Clone, PartialEq, Eq, From, Into, AsRef)]
pub(crate) struct ErgoStateContext(pub(crate) ErgoStateContextInner);

#[pymethods]
impl ErgoStateContext {
    #[new]
    fn new(pre_header: PreHeader, headers: Vec<Header>, parameters: Parameters) -> PyResult<Self> {
        let count = headers.len();
        let headers = HeadersInner::from_vec(headers.into_iter().map(Into::into).collect())
            .map_err(|_| {
                PyValueError::new_err(format!(
                    "Incorrect number of block headers, expected 1..=10 but got {}",
                    count
                ))
            })?;
        Ok(Self(ErgoStateContextInner::new(
            pre_header.into(),
            headers,
            parameters.into(),
        )))
    }
}
