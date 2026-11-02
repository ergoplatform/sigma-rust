use derive_more::{From, Into};
use ergo_lib::{
    chain::transaction::{
        input::prover_result::ProverResult as ProverResultInner, Input as InputInner,
        UnsignedInput as UnsignedInputInner,
    },
    ergotree_ir::serialization::SigmaSerializable,
};
use pyo3::{prelude::*, types::PyType};

use crate::{
    chain::{context_extension::ContextExtension, ergo_box::BoxId},
    errors::{JsonError, SigmaParsingError, SigmaSerializationError},
};

#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub(crate) struct UnsignedInput(UnsignedInputInner);

#[pymethods]
impl UnsignedInput {
    #[pyo3(signature = (box_id, ext=None))]
    #[new]
    fn new(box_id: BoxId, ext: Option<ContextExtension>) -> UnsignedInput {
        UnsignedInput(UnsignedInputInner {
            box_id: box_id.into(),
            extension: ext.unwrap_or_else(|| ContextExtension::new(None)).into(),
        })
    }
    #[getter]
    fn box_id(&self) -> BoxId {
        self.0.box_id.into()
    }
    #[getter]
    fn context_extension(&self) -> ContextExtension {
        self.0.extension.clone().into()
    }
}

#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub(crate) struct Input(InputInner);

#[pymethods]
impl Input {
    #[new]
    fn new(box_id: BoxId, spending_proof: ProverResult) -> Self {
        Input(InputInner::new(box_id.into(), spending_proof.into()))
    }
    #[classmethod]
    fn from_unsigned_input(
        _: &Bound<'_, PyType>,
        unsigned_input: UnsignedInput,
        proof_bytes: Vec<u8>,
    ) -> Self {
        Input(InputInner::from_unsigned_input(
            unsigned_input.into(),
            proof_bytes.into(),
        ))
    }
    #[getter]
    fn box_id(&self) -> BoxId {
        self.0.box_id.into()
    }
    #[getter]
    fn spending_proof(&self) -> ProverResult {
        self.0.spending_proof.clone().into()
    }
    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
}

#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub struct ProverResult(ProverResultInner);

#[pymethods]
impl ProverResult {
    #[classmethod]
    fn from_bytes(_: &Bound<'_, PyType>, b: &[u8]) -> PyResult<Self> {
        ProverResultInner::sigma_parse_bytes(b)
            .map(Self)
            .map_err(SigmaParsingError::from)
            .map_err(Into::into)
    }
    #[getter]
    fn proof(&self) -> Vec<u8> {
        self.0.proof.clone().into()
    }
    #[getter]
    fn extension(&self) -> ContextExtension {
        self.0.extension.clone().into()
    }
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
}
