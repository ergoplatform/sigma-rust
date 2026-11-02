use derive_more::{From, Into};
use pyo3::{exceptions::PyKeyError, prelude::*};

use ergo_lib::ergotree_ir::{
    chain::{context_extension::ContextExtension as ContextExtensionInner, IndexMap},
    serialization::SigmaSerializable,
};

use crate::errors::SigmaSerializationError;

use super::constant::Constant;
#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub struct ContextExtension(ContextExtensionInner);

#[pymethods]
impl ContextExtension {
    #[new]
    #[pyo3(signature=(values=None))]
    pub(crate) fn new(values: Option<IndexMap<u8, Constant>>) -> Self {
        ContextExtension(ContextExtensionInner {
            values: values
                .into_iter()
                .flatten()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        })
    }
    fn __len__(&self) -> usize {
        self.0.values.len()
    }
    fn __contains__(&self, index: u8) -> bool {
        self.0.values.contains_key(&index)
    }
    fn __getitem__(&self, index: u8) -> PyResult<Constant> {
        self.0
            .values
            .get(&index)
            .cloned()
            .map(Into::into)
            .ok_or_else(|| PyKeyError::new_err(format!("{index}")))
    }
    fn __setitem__(&mut self, index: u8, value: Constant) {
        self.0.values.insert(index, value.into());
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
    /// Returns serialized bytes or fails with error if ContextExtension cannot be serialized
    pub fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
}
