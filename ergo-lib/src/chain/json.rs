//! JSON serialization

pub(crate) mod block_header;
pub(crate) mod context_extension;
pub(crate) mod transaction;

use ergotree_interpreter::sigma_protocol::prover::ProofBytes;
use serde::{Deserialize, Serialize};

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
