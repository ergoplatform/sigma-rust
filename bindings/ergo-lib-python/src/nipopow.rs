use derive_more::{From, Into};
use ergo_lib::ergo_nipopow::{
    NipopowProof as NipopowProofInner, NipopowVerifier as NipopowVerifierInner,
    PoPowHeader as PoPowHeaderInner,
};
use pyo3::{prelude::*, types::PyType};
use serde::{Deserialize, Serialize};

use crate::{
    chain::header::{BlockId, Header},
    from_json, to_value_error, JsonError,
};

#[pyclass(frozen, eq, from_py_object)]
#[derive(Clone, From, Into, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct PoPowHeader(PoPowHeaderInner);

#[pymethods]
impl PoPowHeader {
    #[classmethod]
    fn from_json(_: Bound<'_, PyType>, json: Bound<'_, PyAny>) -> PyResult<Self> {
        from_json(json)
    }
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
    #[getter]
    fn header(&self) -> Header {
        self.0.header.clone().into()
    }
    #[getter]
    fn interlinks(&self) -> Vec<BlockId> {
        self.0.interlinks.iter().copied().map(Into::into).collect()
    }
    fn check_interlinks_proof(&self) -> bool {
        self.0.check_interlinks_proof()
    }
}

#[derive(Clone, From, Into, Deserialize, Serialize, PartialEq, Eq)]
#[pyclass(eq, frozen, from_py_object)]
pub(crate) struct NipopowProof(NipopowProofInner);

#[pymethods]
impl NipopowProof {
    #[classmethod]
    fn from_json(_: Bound<'_, PyType>, json: Bound<'_, PyAny>) -> PyResult<Self> {
        from_json(json)
    }
    fn is_better_than(&self, that: &NipopowProof) -> PyResult<bool> {
        self.0.is_better_than(&that.0).map_err(to_value_error)
    }
    #[getter]
    fn suffix_head(&self) -> PoPowHeader {
        self.0.suffix_head.clone().into()
    }
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(&self)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
}

#[pyclass]
pub(crate) struct NipopowVerifier(NipopowVerifierInner);

#[pymethods]
impl NipopowVerifier {
    #[new]
    fn new(genesis_block_id: BlockId) -> Self {
        Self(NipopowVerifierInner::new(genesis_block_id.into()))
    }
    fn best_proof(&self) -> Option<NipopowProof> {
        self.0.best_proof().map(Into::into)
    }
    fn best_chain(&self) -> Vec<Header> {
        self.0.best_chain().into_iter().map(Into::into).collect()
    }
    fn process(&mut self, new_proof: NipopowProof) -> PyResult<()> {
        self.0.process(new_proof.0).map_err(to_value_error)
    }
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "nipopow")?;
    submodule.add_class::<NipopowProof>()?;
    submodule.add_class::<NipopowVerifier>()?;
    submodule.add_class::<PoPowHeader>()?;
    m.add_submodule(&submodule)?;
    submodule
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.nipopow", submodule)?;
    Ok(())
}
