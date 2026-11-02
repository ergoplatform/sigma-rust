use derive_more::{Display, From, Into};
use ergo_lib::{
    ergo_chain_types::EcPoint as EcPointInner, ergotree_ir::serialization::SigmaSerializable,
};
use pyo3::prelude::*;

use crate::to_value_error;

#[pyclass(eq, frozen, str, from_py_object)]
#[derive(PartialEq, Eq, Display, Clone, From, Into)]
pub(crate) struct EcPoint(pub(crate) EcPointInner);

#[pymethods]
impl EcPoint {
    #[new]
    fn new(b: &[u8]) -> PyResult<Self> {
        EcPointInner::sigma_parse_bytes(b)
            .map(Self)
            .map_err(to_value_error)
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
    fn __bytes__(&self) -> Vec<u8> {
        #[allow(clippy::unwrap_used)]
        self.0.sigma_serialize_bytes().unwrap()
    }
}
