use derive_more::{From, Into};
use ergo_lib::{
    ergotree_interpreter::sigma_protocol::{
        prover::hint::{
            RealCommitment as RealCommitmentInner, RealSecretProof as RealSecretProofInner,
            SimulatedCommitment as SimulatedCommitmentInner,
            SimulatedSecretProof as SimulatedSecretProofInner,
        },
        FirstProverMessage,
    },
    ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProofOfKnowledgeTree},
};
use pyo3::{exceptions::PyNotImplementedError, prelude::*};

use crate::{chain::ec_point::EcPoint, sigma_protocol::ProveDlog};

#[derive(From, Into, Clone, PartialEq, Eq)]
#[pyclass(frozen, eq, from_py_object)]
pub(crate) struct RealCommitment(pub(crate) RealCommitmentInner);

#[pymethods]
impl RealCommitment {
    #[getter]
    fn image(&self, _py: Python) -> PyResult<Py<PyAny>> {
        extract_image(_py, &self.0.image)
    }
    #[getter]
    fn commitment(&self, py: Python) -> PyResult<Py<PyAny>> {
        extract_commitment(py, &self.0.commitment)
    }
    #[getter]
    fn position(&self) -> Vec<usize> {
        self.0.position.positions.clone()
    }
}
#[derive(From, Into, Clone, PartialEq, Eq)]
#[pyclass(frozen, eq, from_py_object)]
pub(crate) struct SimulatedCommitment(pub(crate) SimulatedCommitmentInner);

#[pymethods]
impl SimulatedCommitment {
    #[getter]
    fn image(&self, _py: Python) -> PyResult<Py<PyAny>> {
        extract_image(_py, &self.0.image)
    }
    #[getter]
    fn commitment(&self, py: Python) -> PyResult<Py<PyAny>> {
        extract_commitment(py, &self.0.commitment)
    }
    #[getter]
    fn position(&self) -> Vec<usize> {
        self.0.position.positions.clone()
    }
}

#[derive(From, Into, Clone)]
#[pyclass(frozen, from_py_object)]
pub(crate) struct RealSecretProof(pub(crate) RealSecretProofInner);

#[pymethods]
impl RealSecretProof {
    #[getter]
    fn image(&self, _py: Python) -> PyResult<Py<PyAny>> {
        extract_image(_py, &self.0.image)
    }
    #[getter]
    fn challenge(&self) -> Vec<u8> {
        self.0.challenge.clone().into()
    }
    #[getter]
    fn position(&self) -> Vec<usize> {
        self.0.position.positions.clone()
    }
    fn __eq__(&self, other: RealSecretProof) -> PyResult<bool> {
        Err(PyNotImplementedError::new_err("can't compare"))
    }
}

#[derive(From, Into, Clone)]
#[pyclass(frozen, from_py_object)]
pub(crate) struct SimulatedSecretProof(pub(crate) SimulatedSecretProofInner);

#[pymethods]
impl SimulatedSecretProof {
    #[getter]
    fn image(&self, _py: Python) -> PyResult<Py<PyAny>> {
        extract_image(_py, &self.0.image)
    }
    #[getter]
    fn challenge(&self) -> Vec<u8> {
        self.0.challenge.clone().into()
    }
    #[getter]
    fn position(&self) -> Vec<usize> {
        self.0.position.positions.clone()
    }
    fn __eq__(&self, other: RealSecretProof) -> PyResult<bool> {
        Err(PyNotImplementedError::new_err("can't compare"))
    }
}
fn extract_image(py: Python, image: &SigmaBoolean) -> PyResult<Py<PyAny>> {
    match image {
        SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(ref prove_dlog)) => {
            Ok(Py::new(py, ProveDlog::from(prove_dlog.clone()))?.into_any())
        }
        SigmaBoolean::ProofOfKnowledge(_)
        | SigmaBoolean::TrivialProp(_)
        | SigmaBoolean::SigmaConjecture(_) => Err(PyNotImplementedError::new_err(
            "ProveDHTuple is not supported",
        )),
    }
}
fn extract_commitment(py: Python, commitment: &FirstProverMessage) -> PyResult<Py<PyAny>> {
    match commitment {
        FirstProverMessage::FirstDlogProverMessage(ref first_dlog_prover_message) => {
            Ok(Py::new(py, EcPoint::from(*first_dlog_prover_message.a()))?.into_any())
        }
        FirstProverMessage::FirstDhtProverMessage(_) => Err(PyNotImplementedError::new_err(
            "ProveDHTuple is not supported",
        )),
    }
}
