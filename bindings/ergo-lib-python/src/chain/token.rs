use std::str::FromStr;

use derive_more::{From, Into};
use ergo_lib::ergotree_ir::{chain::token, serialization::SigmaSerializable};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyType};

use crate::to_value_error;

use super::ergo_box::BoxId;
#[pyclass(eq, frozen, from_py_object)]
#[derive(PartialEq, Eq, Clone, Copy, From, Into)]
pub(crate) struct Token(pub token::Token);

#[pymethods]
impl Token {
    #[new]
    fn new(token_id: TokenId, amount: u64) -> PyResult<Self> {
        Ok(Token(token::Token {
            token_id: token_id.0,
            amount: amount.try_into().map_err(to_value_error)?,
        }))
    }
    #[getter]
    fn token_id(&self) -> TokenId {
        self.0.token_id.into()
    }
    #[getter]
    fn amount(&self) -> u64 {
        *self.0.amount.as_u64()
    }
    fn __repr__(&self) -> String {
        format!(
            "Token(token_id={:?}, token_amount={})",
            self.0.token_id,
            self.0.amount.as_u64()
        )
    }
}

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(PartialEq, Eq, Clone, Copy, Hash, From)]
pub(crate) struct TokenId(token::TokenId);

#[pymethods]
impl TokenId {
    #[new]
    fn new(val: &Bound<'_, PyAny>) -> PyResult<Self> {
        match val.extract::<&str>() {
            Ok(s) => token::TokenId::from_str(s)
                .map_err(to_value_error)
                .map(Self),
            Err(_) => match val.extract::<&[u8]>() {
                Ok(bytes) => token::TokenId::sigma_parse_bytes(bytes)
                    .map_err(to_value_error)
                    .map(Self),
                Err(_) => Err(PyValueError::new_err(
                    "TokenId.new: missing bytes or str argument",
                )),
            },
        }
    }
    #[classmethod]
    fn from_box_id(_: &Bound<'_, PyType>, box_id: &BoxId) -> Self {
        TokenId(token::TokenId::from(box_id.0))
    }
    fn __bytes__(&self) -> Vec<u8> {
        self.0.into()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}
