use std::str::FromStr;

use data_input::DataInput;
use derive_more::{From, Into};
use ergo_lib::{
    chain::transaction::{
        ergo_transaction::ErgoTransaction, reduced::reduce_tx,
        unsigned::UnsignedTransaction as UnsignedTransactionInner, Transaction as TransactionInner,
        TxId as TxIdInner,
    },
    ergotree_ir::serialization::SigmaSerializable,
};
use input::{Input, ProverResult, UnsignedInput};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyType};
use tx_builder::TxBuilder;

use crate::{
    chain::{
        ergo_box::{ErgoBox, ErgoBoxCandidate},
        ergo_state_context::ErgoStateContext,
    },
    errors::{JsonError, SigmaParsingError, SigmaSerializationError, WalletError},
    from_json, to_value_error,
};

pub mod data_input;
pub mod input;
pub mod tx_builder;

#[pyclass(eq, frozen, hash, str = "{0}", from_py_object)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, From, Into)]
pub(crate) struct TxId(TxIdInner);

#[pymethods]
impl TxId {
    #[new]
    fn new(val: &Bound<'_, PyAny>) -> PyResult<Self> {
        match val.extract::<&str>() {
            Ok(s) => TxIdInner::from_str(s).map_err(to_value_error).map(Self),
            Err(_) => match val.extract::<&[u8]>() {
                Ok(bytes) => TxIdInner::sigma_parse_bytes(bytes)
                    .map_err(to_value_error)
                    .map(Self),
                Err(_) => Err(PyValueError::new_err(
                    "TokenId.new: missing bytes or str argument",
                )),
            },
        }
    }
    fn __bytes__(&self) -> Vec<u8> {
        #[allow(clippy::unwrap_used)]
        self.0.sigma_serialize_bytes().unwrap()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub(crate) struct UnsignedTransaction(pub UnsignedTransactionInner);

#[pymethods]
impl UnsignedTransaction {
    #[new]
    fn new(
        inputs: Vec<UnsignedInput>,
        data_inputs: Vec<DataInput>,
        output_candidates: Vec<ErgoBoxCandidate>,
    ) -> PyResult<Self> {
        UnsignedTransactionInner::new_from_vec(
            inputs.into_iter().map(Into::into).collect(),
            data_inputs.into_iter().map(Into::into).collect(),
            output_candidates.into_iter().map(Into::into).collect(),
        )
        .map(Self)
        .map_err(to_value_error)
    }
    #[getter]
    fn id(&self) -> TxId {
        self.0.id().into()
    }
    #[getter]
    fn inputs(&self) -> Vec<UnsignedInput> {
        self.0.inputs.iter().cloned().map(Into::into).collect()
    }
    #[getter]
    fn data_inputs(&self) -> Vec<DataInput> {
        self.0
            .data_inputs
            .as_ref()
            .into_iter()
            .flatten()
            .copied()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn output_candidates(&self) -> Vec<ErgoBoxCandidate> {
        self.0
            .output_candidates
            .iter()
            .cloned()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn outputs(&self) -> Vec<ErgoBox> {
        self.0.outputs().iter().cloned().map(Into::into).collect()
    }
    #[classmethod]
    fn from_json(_: &Bound<'_, PyType>, s: Bound<'_, PyAny>) -> PyResult<Self> {
        from_json(s).map(Self)
    }
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub(crate) struct Transaction(TransactionInner);

#[pymethods]
impl Transaction {
    #[new]
    fn new(
        inputs: Vec<Input>,
        data_inputs: Vec<DataInput>,
        output_candidates: Vec<ErgoBoxCandidate>,
    ) -> PyResult<Transaction> {
        TransactionInner::new_from_vec(
            inputs.into_iter().map(Into::into).collect(),
            data_inputs.into_iter().map(Into::into).collect(),
            output_candidates.into_iter().map(Into::into).collect(),
        )
        .map(Self)
        .map_err(to_value_error)
    }
    #[classmethod]
    fn from_unsigned_tx(
        _: &Bound<'_, PyType>,
        unsigned_tx: UnsignedTransaction,
        proofs: Vec<Vec<u8>>,
    ) -> PyResult<Self> {
        TransactionInner::from_unsigned_tx(
            unsigned_tx.into(),
            proofs.into_iter().map(Into::into).collect(),
        )
        .map(Self)
        .map_err(to_value_error)
    }
    #[classmethod]
    fn from_json(_: &Bound<'_, PyType>, json: Bound<'_, PyAny>) -> PyResult<Self> {
        from_json(json).map(Self)
    }
    #[classmethod]
    fn from_bytes(_: &Bound<'_, PyType>, b: &[u8]) -> PyResult<Self> {
        TransactionInner::sigma_parse_bytes(b)
            .map(Self)
            .map_err(SigmaParsingError::from)
            .map_err(Into::into)
    }
    #[getter]
    fn id(&self) -> TxId {
        self.0.id().into()
    }
    #[getter]
    fn inputs(&self) -> Vec<Input> {
        self.0.inputs.iter().cloned().map(Into::into).collect()
    }
    #[getter]
    fn data_inputs(&self) -> Vec<DataInput> {
        self.0
            .data_inputs
            .as_ref()
            .into_iter()
            .flatten()
            .cloned()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn output_candidates(&self) -> Vec<ErgoBoxCandidate> {
        self.0
            .output_candidates
            .iter()
            .cloned()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn outputs(&self) -> Vec<ErgoBox> {
        self.0.outputs.iter().cloned().map(Into::into).collect()
    }

    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }

    fn json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
}
#[pyclass(eq, frozen, from_py_object)]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub(crate) struct ReducedTransaction(ergo_lib::chain::transaction::reduced::ReducedTransaction);

#[pymethods]
impl ReducedTransaction {
    #[classmethod]
    fn from_unsigned_tx(
        _: &Bound<'_, PyType>,
        unsigned_tx: &UnsignedTransaction,
        boxes_to_spend: Vec<ErgoBox>,
        data_boxes: Vec<ErgoBox>,
        state_context: ErgoStateContext,
    ) -> PyResult<ReducedTransaction> {
        let boxes_to_spend = boxes_to_spend.into_iter().map(Into::into).collect();
        let data_boxes = data_boxes.into_iter().map(Into::into).collect();
        let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
            unsigned_tx.0.clone(),
            boxes_to_spend,
            data_boxes,
        )
        .map_err(to_value_error)?;
        reduce_tx(tx_context, &state_context.into())
            .map_err(Into::into)
            .map_err(WalletError)
            .map_err(Into::into)
            .map(ReducedTransaction::from)
    }

    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }

    #[classmethod]
    fn from_bytes(_: &Bound<'_, PyType>, b: &[u8]) -> PyResult<ReducedTransaction> {
        ergo_lib::chain::transaction::reduced::ReducedTransaction::sigma_parse_bytes(b)
            .map(ReducedTransaction)
            .map_err(SigmaParsingError::from)
            .map_err(Into::into)
    }

    fn unsigned_tx(&self) -> UnsignedTransaction {
        self.0.unsigned_tx.clone().into()
    }
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "transaction")?;
    submodule.add_class::<UnsignedInput>()?;
    submodule.add_class::<Input>()?;
    submodule.add_class::<DataInput>()?;
    submodule.add_class::<ProverResult>()?;
    submodule.add_class::<UnsignedTransaction>()?;
    submodule.add_class::<Transaction>()?;
    submodule.add_class::<ReducedTransaction>()?;
    submodule.add_class::<TxBuilder>()?;
    submodule.add_class::<TxId>()?;
    m.add_submodule(&submodule)?;
    m.py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.transaction", submodule)?;
    Ok(())
}
