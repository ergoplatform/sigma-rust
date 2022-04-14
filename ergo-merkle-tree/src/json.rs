use crate::batchmerkleproof::BatchMerkleProof;
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
            match hash.len() {
                0 => None,
                _ => Some(
                    hash.try_into()
                        .map_err(|_| MerkleProofFromJsonError::LengthError)?,
                ),
            },
            node.1,
        ))
    }
}

impl From<LevelNode> for LevelNodeJson {
    fn from(node: LevelNode) -> Self {
        Self(
            node.0
                .map(|hash| base16::encode_lower(&hash))
                .unwrap_or_else(String::new),
            node.1,
        )
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
        let levels: Result<Vec<LevelNode>, Self::Error> =
            proof.levels.into_iter().map(LevelNode::try_from).collect();
        Ok(Self {
            leaf_data,
            levels: levels?,
        })
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

#[derive(Serialize, Deserialize)]
struct IndexJson {
    index: usize,
    digest: String,
}

impl From<(usize, [u8; 32])> for IndexJson {
    fn from(index: (usize, [u8; 32])) -> IndexJson {
        IndexJson {
            index: index.0,
            digest: base16::encode_lower(&index.1),
        }
    }
}

impl TryFrom<IndexJson> for (usize, [u8; 32]) {
    type Error = MerkleProofFromJsonError;
    fn try_from(index: IndexJson) -> Result<(usize, [u8; 32]), Self::Error> {
        let digest = base16::decode(&index.digest)?
            .try_into()
            .map_err(|_| MerkleProofFromJsonError::LengthError)?;
        Ok((index.index, digest))
    }
}

#[derive(Serialize, Deserialize)]
pub struct BatchMerkleProofJson {
    indices: Vec<IndexJson>,
    proofs: Vec<LevelNodeJson>,
}

impl std::convert::TryFrom<BatchMerkleProofJson> for crate::batchmerkleproof::BatchMerkleProof {
    type Error = MerkleProofFromJsonError;
    fn try_from(
        json: BatchMerkleProofJson,
    ) -> Result<crate::batchmerkleproof::BatchMerkleProof, Self::Error> {
        let indices: Result<Vec<(usize, [u8; 32])>, Self::Error> =
            json.indices.into_iter().map(IndexJson::try_into).collect();
        let proofs: Result<Vec<LevelNode>, Self::Error> =
            json.proofs.into_iter().map(LevelNode::try_from).collect();
        Ok(BatchMerkleProof {
            indices: indices?,
            proofs: proofs?,
        })
    }
}

impl From<crate::batchmerkleproof::BatchMerkleProof> for BatchMerkleProofJson {
    fn from(proof: crate::batchmerkleproof::BatchMerkleProof) -> BatchMerkleProofJson {
        let indices = proof.indices.into_iter().map(IndexJson::from).collect();
        let proofs = proof.proofs.into_iter().map(LevelNodeJson::from).collect();
        BatchMerkleProofJson { indices, proofs }
    }
}
