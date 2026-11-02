use derive_more::{AsRef, From, Into};
use ergo_lib::{
    ergotree_interpreter::sigma_protocol::prover::hint::{Hint, HintsBag as HintsBagInner},
    wallet::TransactionHintsBag as TransactionHintsBagInner,
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyType};

use crate::errors::JsonError;

use super::hints::{RealCommitment, RealSecretProof, SimulatedCommitment, SimulatedSecretProof};

#[pyclass(from_py_object)]
#[derive(From, Into, Clone)]
pub(crate) struct HintsBag(HintsBagInner);

#[pymethods]
impl HintsBag {
    #[new]
    fn new() -> Self {
        Self(HintsBagInner::empty())
    }
    fn add_commitment(&mut self, commitment: &Bound<'_, PyAny>) -> PyResult<()> {
        match commitment.extract::<RealCommitment>() {
            Ok(real) => self.0.add_hint(Hint::CommitmentHint(real.0.into())),
            Err(_) => match commitment.extract::<SimulatedCommitment>() {
                Ok(simulated) => self.0.add_hint(Hint::CommitmentHint(simulated.0.into())),
                Err(e) => {
                    return Err(PyValueError::new_err(
                        "Expected RealCommitment or SimulatedCommitment",
                    ))
                }
            },
        }
        Ok(())
    }
    fn add_proof(&mut self, proof: &Bound<'_, PyAny>) -> PyResult<()> {
        match proof.extract::<RealSecretProof>() {
            Ok(real) => self.0.add_hint(Hint::SecretProven(real.0.into())),
            Err(_) => match proof.extract::<SimulatedSecretProof>() {
                Ok(simulated) => self.0.add_hint(Hint::SecretProven(simulated.0.into())),
                Err(e) => {
                    return Err(PyValueError::new_err(
                        "Expected RealSecretProof or SimulatedSecretProof",
                    ))
                }
            },
        }
        Ok(())
    }
    #[getter]
    fn real_commitments(&self) -> Vec<RealCommitment> {
        self.0
            .real_commitments()
            .into_iter()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn simulated_commitments(&self) -> Vec<SimulatedCommitment> {
        self.0
            .simulated_commitments()
            .into_iter()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn real_proofs(&self) -> Vec<RealSecretProof> {
        self.0.real_proofs().into_iter().map(Into::into).collect()
    }
    #[getter]
    fn simulated_proofs(&self) -> Vec<SimulatedSecretProof> {
        self.0
            .simulated_proofs()
            .into_iter()
            .map(Into::into)
            .collect()
    }
    /// Create a new HintsBag with private data (OwnCommitment) removed. This is suitable for sending to other co-signers as HintsBag could contain private data by default
    // TODO: consider adding to other bindings as well to prevent footguns
    fn without_secrets(&self) -> Self {
        let mut new_bag = HintsBagInner::empty();
        self.0
            .simulated_commitments()
            .into_iter()
            .for_each(|simulated| new_bag.add_hint(Hint::CommitmentHint(simulated.into())));
        self.0
            .real_commitments()
            .into_iter()
            .for_each(|real| new_bag.add_hint(Hint::CommitmentHint(real.into())));
        self.0
            .proofs()
            .into_iter()
            .for_each(|proof| new_bag.add_hint(Hint::SecretProven(proof)));
        new_bag.into()
    }
    fn json(&self) -> PyResult<String> {
        self.without_secrets().private_json()
    }
    fn private_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
}

#[pyclass(from_py_object)]
#[derive(Clone, From, Into, AsRef)]
pub(crate) struct TransactionHintsBag(TransactionHintsBagInner);

#[pymethods]
impl TransactionHintsBag {
    #[new]
    fn new() -> Self {
        TransactionHintsBag(TransactionHintsBagInner::empty())
    }
    #[classmethod]
    fn from_json(_: &Bound<'_, PyType>, s: &str) -> PyResult<Self> {
        serde_json::from_str(s)
            .map(Self)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
    fn add_hints_for_input(&mut self, index: usize, hints_bag: HintsBag) {
        self.0.add_hints_for_input(index, hints_bag.0);
    }
    fn all_hints_for_input(&self, index: usize) -> HintsBag {
        HintsBag::from(self.0.all_hints_for_input(index))
    }
    // TODO: add private/json here as well?
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
}
