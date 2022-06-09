/// Marker trait for response data types
pub trait NodeResponse {}

impl NodeResponse for ergo_merkle_tree::MerkleProof {}
impl NodeResponse for ergo_chain_types::Header {}
