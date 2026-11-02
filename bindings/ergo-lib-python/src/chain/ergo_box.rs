use std::{collections::HashMap, str::FromStr};

use derive_more::{From, Into};
use ergo_lib::{
    chain::ergo_box::box_builder::ErgoBoxCandidateBuilder,
    ergotree_ir::{
        chain::ergo_box::{
            self, box_value::BoxValue, NonMandatoryRegisters as NonMandatoryRegistersInner,
        },
        serialization::SigmaSerializable,
    },
};
use pyo3::{
    exceptions::{PyKeyError, PyValueError},
    prelude::*,
    types::{PyDict, PyType},
};
use serde::{Deserialize, Serialize};
use serde_pyobject::from_pyobject;

use crate::{
    ergo_tree::ErgoTree,
    errors::{JsonError, RegisterValueError, SigmaParsingError, SigmaSerializationError},
    from_json, to_value_error,
    transaction::TxId,
};

use super::{address::Address, constant::Constant, token::Token};

#[pyclass(eq, frozen, hash, ord, from_py_object)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(u8)]
pub(crate) enum NonMandatoryRegisterId {
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
}

impl From<ergo_box::NonMandatoryRegisterId> for NonMandatoryRegisterId {
    fn from(value: ergo_box::NonMandatoryRegisterId) -> Self {
        match value {
            ergo_box::NonMandatoryRegisterId::R4 => Self::R4,
            ergo_box::NonMandatoryRegisterId::R5 => Self::R5,
            ergo_box::NonMandatoryRegisterId::R6 => Self::R6,
            ergo_box::NonMandatoryRegisterId::R7 => Self::R7,
            ergo_box::NonMandatoryRegisterId::R8 => Self::R8,
            ergo_box::NonMandatoryRegisterId::R9 => Self::R9,
        }
    }
}

impl From<NonMandatoryRegisterId> for ergo_box::NonMandatoryRegisterId {
    fn from(id: NonMandatoryRegisterId) -> ergo_box::NonMandatoryRegisterId {
        #[allow(clippy::unwrap_used)]
        ergo_box::NonMandatoryRegisterId::try_from(id as i8).unwrap()
    }
}

/// Identifier of an :class:`ErgoBox`
#[pyclass(str = "{0}", eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, Copy, From, Into)]
pub(crate) struct BoxId(pub ergo_box::BoxId);

#[pymethods]
impl BoxId {
    #[new]
    fn new(val: &Bound<'_, PyAny>) -> PyResult<Self> {
        match val.extract::<&str>() {
            Ok(s) => ergo_box::BoxId::from_str(s)
                .map_err(to_value_error)
                .map(Self),
            Err(_) => match val.extract::<&[u8]>() {
                Ok(bytes) => ergo_box::BoxId::sigma_parse_bytes(bytes)
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

#[pyclass(eq, from_py_object)]
#[derive(Clone, PartialEq, Eq, From, Into, Debug)]
pub(crate) struct ErgoBoxCandidate(ergo_box::ErgoBoxCandidate);

#[pymethods]
impl ErgoBoxCandidate {
    #[allow(non_snake_case)]
    #[classattr]
    fn SAFE_USER_MIN() -> u64 {
        *BoxValue::SAFE_USER_MIN.as_u64()
    }
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature=(*, value, script, creation_height, tokens=None, registers=None, mint_token= None, mint_token_name = None, mint_token_desc=None, mint_token_decimals=None))]
    fn new(
        value: u64,
        script: &Bound<'_, PyAny>,
        creation_height: u32,
        tokens: Option<Vec<Token>>,
        registers: Option<HashMap<NonMandatoryRegisterId, Constant>>,
        mint_token: Option<Token>,
        mint_token_name: Option<&str>,
        mint_token_desc: Option<&str>,
        mint_token_decimals: Option<usize>,
    ) -> PyResult<Self> {
        let tree = match script.extract::<ErgoTree>() {
            Ok(tree) => tree,
            Err(e) => match script.extract::<Address>() {
                Ok(addr) => addr.ergo_tree()?,
                Err(e) => return Err(PyValueError::new_err("expected ErgoTree or Address")),
            },
        };
        let mut builder = ErgoBoxCandidateBuilder::new(
            BoxValue::new(value).map_err(to_value_error)?,
            tree.0,
            creation_height,
        );
        for token in tokens.into_iter().flatten() {
            builder.add_token(token.into());
        }
        for (id, value) in registers.into_iter().flatten() {
            builder.set_register_value(id.into(), value.into());
        }
        if let Some(mint_token) = mint_token {
            (|| {
                builder.mint_token(
                    mint_token.into(),
                    mint_token_name?.into(),
                    mint_token_desc?.into(),
                    mint_token_decimals?,
                );
                Some(())
            })()
            .ok_or_else(|| {
                PyValueError::new_err(
                    "Expected mint_token_name, mint_token_desc, mint_token_decimals",
                )
            })?;
        }
        builder.build().map(Self).map_err(to_value_error)
    }
    #[getter]
    fn value(&self) -> u64 {
        *self.0.value.as_u64()
    }
    #[getter]
    fn creation_height(&self) -> u32 {
        self.0.creation_height
    }
    #[getter]
    fn tokens(&self) -> Vec<Token> {
        self.0
            .tokens
            .iter()
            .flatten()
            .copied()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn additional_registers(&self) -> NonMandatoryRegisters {
        NonMandatoryRegisters(self.0.additional_registers.clone())
    }
    #[getter]
    fn ergo_tree(&self) -> ErgoTree {
        self.0.ergo_tree.clone().into()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into, Deserialize, Serialize)]
pub(crate) struct ErgoBox(pub ergo_box::ErgoBox);

#[pymethods]
impl ErgoBox {
    #[new]
    fn new(dict: Bound<'_, PyDict>) -> PyResult<Self> {
        from_pyobject::<ergo_box::ErgoBox, PyDict>(dict)
            .map(Self)
            .map_err(to_value_error)
    }
    #[getter]
    fn box_id(&self) -> BoxId {
        self.0.box_id().into()
    }
    #[getter]
    fn value(&self) -> u64 {
        *self.0.value.as_u64()
    }
    #[getter]
    fn creation_height(&self) -> u32 {
        self.0.creation_height
    }
    #[getter]
    fn tokens(&self) -> Vec<Token> {
        self.0
            .tokens
            .iter()
            .flatten()
            .copied()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn additional_registers(&self) -> NonMandatoryRegisters {
        NonMandatoryRegisters(self.0.additional_registers.clone())
    }
    #[getter]
    fn ergo_tree(&self) -> ErgoTree {
        self.0.ergo_tree.clone().into()
    }
    #[getter]
    fn transaction_id(&self) -> TxId {
        self.0.transaction_id.into()
    }
    #[getter]
    fn index(&self) -> u16 {
        self.0.index
    }
    #[pyo3(text_signature = "(self) -> str")]
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
    #[classmethod]
    fn from_box_candidate(
        _: &Bound<'_, PyType>,
        candidate: ErgoBoxCandidate,
        tx_id: TxId,
        index: u16,
    ) -> PyResult<Self> {
        ergo_box::ErgoBox::from_box_candidate(&candidate.into(), tx_id.into(), index)
            .map(Into::into)
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
    #[classmethod]
    fn from_json(_: &Bound<'_, PyType>, json: Bound<'_, PyAny>) -> PyResult<Self> {
        from_json(json)
    }
    #[classmethod]
    fn from_bytes(_: &Bound<'_, PyType>, b: &[u8]) -> PyResult<Self> {
        ergo_box::ErgoBox::sigma_parse_bytes(b)
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
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyclass(eq, frozen, skip_from_py_object)]
#[derive(PartialEq, Eq, From, Into)]
pub(crate) struct NonMandatoryRegisters(NonMandatoryRegistersInner);

#[pymethods]
impl NonMandatoryRegisters {
    fn __len__(&self) -> usize {
        self.0.len()
    }
    fn __getitem__(&self, index: NonMandatoryRegisterId) -> PyResult<Constant> {
        self.0
            .get_constant(index.into())
            .map_err(RegisterValueError::from)
            .map_err(PyErr::from)?
            .ok_or_else(|| PyKeyError::new_err(format!("{index:?} out of bounds")))
            .map(Into::into)
    }
    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
}
