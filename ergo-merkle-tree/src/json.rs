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

#[derive(Serialize, Deserialize)]
struct Index {
    index: usize,
    digest: [i8; crate::HASH_SIZE],
}
#[derive(Serialize, Deserialize)]
struct ProofNode {
    digest: Vec<i8>, // a proof node's hash can be empty, so we use a Vec instead of a fixed-size slice
    side: crate::NodeSide,
}

#[derive(Serialize, Deserialize)]
pub struct BatchMerkleProofJson {
    indices: Vec<Index>,
    proofs: Vec<ProofNode>,
}

impl std::convert::TryFrom<BatchMerkleProofJson> for crate::batchmerkleproof::BatchMerkleProof {
    type Error = MerkleProofFromJsonError;
    fn try_from(
        json: BatchMerkleProofJson,
    ) -> Result<crate::batchmerkleproof::BatchMerkleProof, Self::Error> {
        let mut indices = vec![];

        for index in json.indices {
            let digest = index
                .digest
                .iter()
                .map(|&x| x as u8)
                .collect::<Vec<u8>>()
                .try_into()
                .map_err(|_| MerkleProofFromJsonError::LengthError)?;
            indices.push((index.index, digest));
        }
        let mut proofs = vec![];
        for proof in json.proofs {
            #[allow(clippy::unwrap_used)]
            // unwrapping into a [u8; 32] is safe since we check the length
            let level_node = match proof.digest.len() {
                0 => crate::LevelNode::empty_node(proof.side),
                32 => crate::LevelNode::new(
                    proof
                        .digest
                        .iter()
                        .map(|&x| x as u8)
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap(),
                    proof.side,
                ),
                _ => return Err(MerkleProofFromJsonError::LengthError),
            };
            proofs.push(level_node);
        }

        Ok(crate::batchmerkleproof::BatchMerkleProof::new(
            indices, proofs,
        ))
    }
}

impl From<crate::batchmerkleproof::BatchMerkleProof> for BatchMerkleProofJson {
    fn from(proof: crate::batchmerkleproof::BatchMerkleProof) -> BatchMerkleProofJson {
        let indices = proof
            .indices
            .into_iter()
            .map(|(index, digest)| Index {
                index,
                digest: digest
                    .iter()
                    .map(|&x| x as i8)
                    .collect::<Vec<i8>>()
                    .try_into()
                    .unwrap(),
            })
            .collect();
        let proofs = proof
            .proofs
            .into_iter()
            .map(|node| ProofNode {
                digest: node
                    .0
                    .into_iter()
                    .flat_map(|hash| hash.into_iter().map(|x| x as i8))
                    .collect(),
                side: node.1,
            })
            .collect();
        BatchMerkleProofJson { indices, proofs }
    }
}
