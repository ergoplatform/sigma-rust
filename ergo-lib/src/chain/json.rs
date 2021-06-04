//! JSON serialization

pub(crate) mod block_header;
pub(crate) mod context_extension;
pub(crate) mod ergo_box;
pub(crate) mod ergo_tree;
pub(crate) mod transaction;

use ergotree_interpreter::sigma_protocol::prover::ProofBytes;
use serde::Serializer;
use serde::{Deserialize, Serialize};

pub fn serialize_bytes<S, T>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    serializer.serialize_str(&base16::encode_lower(bytes.as_ref()))
}

#[cfg_attr(
    feature = "json",
    derive(Serialize, Deserialize),
    serde(into = "String", try_from = "String"),
    serde(remote = "ProofBytes")
)]
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum ProofBytesSerde {
    /// Empty proof
    Empty,
    /// Non-empty proof
    Some(Vec<u8>),
}
