use std::str::FromStr;

use derive_more::{From, Into};
use ergo_lib::ergo_chain_types::{
    BlockId as InnerBlockId, Digest32, Header as InnerHeader, PreHeader as InnerPreHeader,
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyType};
use serde::Deserialize;
use sigma_ser::ScorexSerializable;

use crate::{from_json, to_value_error};

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(PartialEq, Eq, Clone, Copy, Hash, From, Into)]
pub(crate) struct BlockId(InnerBlockId);

#[pymethods]
impl BlockId {
    #[new]
    fn new(val: &Bound<'_, PyAny>) -> PyResult<Self> {
        match val.extract::<&str>() {
            Ok(s) => InnerBlockId::from_str(s).map_err(to_value_error).map(Self),
            Err(_) => match val.extract::<&[u8]>() {
                Ok(bytes) => Digest32::scorex_parse_bytes(bytes)
                    .map_err(to_value_error)
                    .map(InnerBlockId)
                    .map(Self),
                Err(_) => Err(PyValueError::new_err(
                    "BlockId.new: missing bytes or str argument",
                )),
            },
        }
    }
    fn __bytes__(&self) -> Vec<u8> {
        self.0 .0 .0.into()
    }
    fn __str__(&self) -> String {
        self.0.to_string()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into, Deserialize)]
pub struct Header(InnerHeader);

#[pymethods]
impl Header {
    #[classmethod]
    fn from_json(_: &Bound<'_, PyType>, json: Bound<'_, PyAny>) -> PyResult<Self> {
        from_json(json)
    }
    #[getter]
    fn version(&self) -> u8 {
        self.0.version
    }
    #[getter]
    fn id(&self) -> BlockId {
        BlockId(self.0.id)
    }
    #[getter]
    fn parent_id(&self) -> BlockId {
        BlockId(self.0.parent_id)
    }
    #[getter]
    fn ad_proofs_root(&self) -> [u8; 32] {
        self.0.ad_proofs_root.0
    }
    #[getter]
    fn state_root(&self) -> [u8; 33] {
        self.0.state_root.0
    }
    #[getter]
    fn transaction_root(&self) -> [u8; 32] {
        self.0.transaction_root.0
    }
    #[getter]
    fn timestamp(&self) -> u64 {
        self.0.timestamp
    }
    #[getter]
    fn n_bits(&self) -> u32 {
        self.0.n_bits
    }
    #[getter]
    fn height(&self) -> u32 {
        self.0.height
    }
    #[getter]
    fn extension_root(&self) -> [u8; 32] {
        self.0.extension_root.0
    }
}

/// Block header with the current `spendingTransaction`, that can be predicted
/// by a miner before it's formation
#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub struct PreHeader(InnerPreHeader);

#[pymethods]
impl PreHeader {
    #[new]
    fn new(header: Header) -> PreHeader {
        PreHeader(InnerPreHeader::from(header.0))
    }
}
