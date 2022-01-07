use crate::{LevelNode, MerkleProof, NodeSide};
use serde::{Deserialize, Serialize};
use thiserror::Error;
/// Json Representation of a LevelNode. First field must be valid base16
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelNodeJson(String, NodeSide);

impl std::convert::TryFrom<LevelNodeJson> for LevelNode {
    type Error = MerkleProofFromJsonError;
    fn try_from(node: LevelNodeJson) -> Result<Self, Self::Error> {
        let hash = base16::decode(&node.0)?;
        Ok(LevelNode(
            hash.try_into()
                .map_err(|_| MerkleProofFromJsonError::LengthError)?,
            node.1,
        ))
    }
}

impl From<LevelNode> for LevelNodeJson {
    fn from(node: LevelNode) -> Self {
        Self(base16::encode_lower(&node.0), node.1)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MerkleProofJson {
    #[serde(rename = "leafData")]
    /// Leaf Data. Must be a valid base16 string and decode to a 32 byte hash
    pub leaf_data: String,
    /// Level Nodes used for Merkle Proof verification
    pub levels: Vec<LevelNodeJson>,
}

/// Error deserializing MerkleProof from Json
#[cfg(feature = "json")]
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum MerkleProofFromJsonError {
    /// Base16 decoding has failed
    #[error("Failed to decode base16 string")]
    DecodeError(#[from] base16::DecodeError),
    /// Invalid Length (expected 32 bytes)
    #[error("Invalid Length. Hashes and Leaf data must be 32 bytes in size")]
    LengthError,
}

#[cfg(feature = "json")]
impl std::convert::TryFrom<crate::json::MerkleProofJson> for MerkleProof {
    type Error = MerkleProofFromJsonError;
    fn try_from(proof: crate::json::MerkleProofJson) -> Result<Self, Self::Error> {
        let leaf_data = base16::decode(&proof.leaf_data)?;
        let leaf_data: [u8; 32] = leaf_data
            .try_into()
            .map_err(|_| MerkleProofFromJsonError::LengthError)?;
        let mut levels = Vec::with_capacity(proof.levels.len());
        for node in proof.levels {
            let node: LevelNode = node.try_into()?;
            levels.push(node);
        }
        Ok(Self { leaf_data, levels })
    }
}
#[cfg(feature = "json")]
impl From<MerkleProof> for MerkleProofJson {
    fn from(proof: MerkleProof) -> Self {
        let levels: Vec<crate::json::LevelNodeJson> =
            proof.levels.into_iter().map(Into::into).collect();
        let leaf_data = base16::encode_lower(&proof.leaf_data);
        Self { leaf_data, levels }
    }
}
