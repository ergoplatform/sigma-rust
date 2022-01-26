//! Merkle Proof verification
use ergo_lib::ergo_merkle_tree;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// A MerkleProof type. Given leaf data and levels (bottom-upwards), the root hash can be computed and validated
pub struct MerkleProof(ergo_merkle_tree::MerkleProof);

/// A level node in a merkle proof
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct LevelNode(ergo_merkle_tree::LevelNode);

#[wasm_bindgen]
impl LevelNode {
    /// Creates a new LevelNode from a 32 byte hash and side that the node belongs on in the tree. Fails if the digest is not 32 bytes
    pub fn new(hash: &[u8], side: u8) -> Result<LevelNode, JsValue> {
        Ok(Self(ergo_merkle_tree::LevelNode::new(
            (*hash)
                .try_into()
                .map_err(|_| "Digest is not 32 bytes in size")?,
            side.try_into()?,
        )))
    }
    /// Returns the associated digest (hash) with this node. Returns an empty array if there's no hash
    #[wasm_bindgen(getter)]
    pub fn digest(&self) -> Vec<u8> {
        match self.0 .0 {
            Some(hash) => hash[0..].to_owned(),
            None => vec![],
        }
    }
    /// Returns the associated side with this node (0 = Left, 1 = Right)
    #[wasm_bindgen(getter)]
    pub fn side(&self) -> u8 {
        self.0 .1 as u8
    }
}

#[wasm_bindgen]
impl MerkleProof {
    /// Creates a new merkle proof with given leaf data and level data (bottom-upwards)
    /// You can verify it against a Blakeb256 root hash by using [`Self::valid()`]
    /// Add a node by using [`Self::add_node()`]
    /// Each digest on the level must be exactly 32 bytes
    pub fn new(leaf_data: &[u8]) -> Result<MerkleProof, wasm_bindgen::JsValue> {
        Ok(Self(
            ergo_merkle_tree::MerkleProof::new(leaf_data, &[]).map_err(|err| err.to_string())?,
        )) // There are issues with wasm when trying to pass an array of structs, so it's better to use add_node instead
    }

    /// Adds a new node to the MerkleProof above the current nodes
    pub fn add_node(&mut self, level: &LevelNode) {
        self.0.add_node(level.0);
    }

    /// Validates the Merkle proof against the root hash
    pub fn valid(&self, expected_root: &[u8]) -> bool {
        self.0.valid(expected_root)
    }
}

#[wasm_bindgen]
/// Decodes a base16 string into an array of bytes
pub fn base16_decode(data: &str) -> Result<Vec<u8>, JsValue> {
    base16::decode(&data).map_err(|_| "Failed to decode base16 input".into())
}
