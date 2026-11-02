use derive_more::{From, Into};
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog as ProveDlogInner;
use pyo3::prelude::*;

use crate::chain::ec_point::EcPoint;

#[pyclass(eq, frozen, from_py_object)]
#[derive(PartialEq, Eq, From, Into, Clone)]
pub(crate) struct ProveDlog(pub(crate) ProveDlogInner);

#[pymethods]
impl ProveDlog {
    #[new]
    fn new(ec_point: EcPoint) -> Self {
        ProveDlogInner::new(ec_point.into()).into()
    }
    #[getter]
    fn h(&self) -> EcPoint {
        (*self.0.h).into()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "sigma_protocol")?;
    submodule.add_class::<ProveDlog>()?;
    m.add_submodule(&submodule)?;
    m.py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.sigma_protocol", submodule)?;
    Ok(())
}
