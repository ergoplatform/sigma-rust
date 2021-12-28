use crate::{LevelNode, NodeSide};
use serde::{Deserialize, Serialize};
/// Json Representation of a LevelNode. First field must be valid base16
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelNodeJson(String, NodeSide);

impl std::convert::TryFrom<LevelNodeJson> for LevelNode {
    type Error = base16::DecodeError;
    fn try_from(node: LevelNodeJson) -> Result<Self, Self::Error> {
        let hash = base16::decode(&node.0)?;
        if hash.len() != 32 {
            // TODO: Should we accept hashes that are not 32 bytes in size? Also this error could probably be cleaner with a custom error type
            return Err(base16::DecodeError::InvalidLength {
                length: node.0.len(),
            });
        }
        Ok(LevelNode(hash.try_into().unwrap(), node.1))
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
