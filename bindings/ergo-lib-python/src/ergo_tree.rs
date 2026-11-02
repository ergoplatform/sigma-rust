use derive_more::{From, Into};
use ergo_lib::ergotree_ir::{ergo_tree, serialization::SigmaSerializable};
use pyo3::{prelude::*, types::PyType};

use crate::{
    errors::{SigmaParsingError, SigmaSerializationError},
    to_value_error,
};

use super::chain::constant::Constant;

#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub(crate) struct ErgoTree(pub ergo_tree::ErgoTree);

#[pymethods]
impl ErgoTree {
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
    fn constants(&self) -> PyResult<Vec<Constant>> {
        self.0
            .get_constants()
            .map_err(to_value_error)
            .map(|constants| constants.into_iter().map(Into::into).collect())
    }
    fn with_constant(&self, index: usize, constant: Constant) -> PyResult<Self> {
        self.0
            .clone()
            .with_constant(index, constant.into())
            .map(Self)
            .map_err(to_value_error)
    }
    #[classmethod]
    fn from_bytes(_: &Bound<'_, PyType>, b: &[u8]) -> PyResult<Self> {
        ergo_tree::ErgoTree::sigma_parse_bytes(b)
            .map(Self)
            .map_err(SigmaParsingError::from)
            .map_err(Into::into)
    }
    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "ergo_tree")?;
    submodule.add_class::<ErgoTree>()?;
    m.add_submodule(&submodule)?;
    m.py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.ergo_tree", submodule)?;
    Ok(())
}
