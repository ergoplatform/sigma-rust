/// Raw bindings for ergo_merkle_tree::MerkleProof
use crate::{
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};
pub use ergo_merkle_tree::NodeSide;
use std::convert::TryInto;

/// Merkle Proof type
pub struct MerkleProof(ergo_merkle_tree::MerkleProof);

pub type MerkleProofPtr = *mut MerkleProof;
pub type ConstMerkleProofPtr = *const MerkleProof;

/// Creates a new MerkleProof . Use merkleproof_add_node for adding level nodes
pub unsafe fn merkleproof_new(
    leaf_data: &[u8],
    proof_out: *mut MerkleProofPtr,
) -> Result<(), Error> {
    let proof_out = mut_ptr_as_mut(proof_out, "proof_out")?;
    *proof_out = Box::into_raw(Box::new(MerkleProof(ergo_merkle_tree::MerkleProof::new(
        leaf_data,
        &[],
    )?)));
    Ok(())
}

/// Adds a LevelNode to a MerkleProof
pub unsafe fn merkleproof_add_node(
    proof: MerkleProofPtr,
    hash: &[u8],
    side: NodeSide,
) -> Result<(), Error> {
    let proof = mut_ptr_as_mut(proof, "proof")?;
    let node = ergo_merkle_tree::LevelNode::new(hash.try_into()?, side);
    proof.0.add_node(node);

    Ok(())
}

/// Checks the MerkleProof against a root hash
pub unsafe fn merkleproof_valid(
    proof: ConstMerkleProofPtr,
    expected_root: &[u8],
) -> Result<bool, Error> {
    let proof = const_ptr_as_ref(proof, "proof")?;
    Ok(proof.0.valid(expected_root))
}

/// Checks the Merkleproof against a base16 root hash
pub unsafe fn merkleproof_valid_base16(
    proof: ConstMerkleProofPtr,
    expected_root: &str,
) -> Result<bool, Error> {
    let proof = const_ptr_as_ref(proof, "proof")?;
    Ok(proof.0.valid_base16(expected_root)?)
}

/// Deserializes MerkleProof from JSON
pub unsafe fn merkleproof_from_json(
    json: &str,
    proof_out: *mut MerkleProofPtr,
) -> Result<(), Error> {
    let proof_out = mut_ptr_as_mut(proof_out, "proof_out")?;
    *proof_out = Box::into_raw(Box::new(serde_json::from_str(json).map(MerkleProof)?));
    Ok(())
}

/// Serializes a MerkleProof to Json
pub unsafe fn merkleproof_to_json(proof: ConstMerkleProofPtr) -> Result<String, Error> {
    let proof = const_ptr_as_ref(proof, "proof")?;
    serde_json::to_string(&proof.0).map_err(Error::from)
}
