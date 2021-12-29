use crate::MerkleProofFromJsonError;
use crate::{LevelNode, NodeSide};
use serde::{Deserialize, Serialize};
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
