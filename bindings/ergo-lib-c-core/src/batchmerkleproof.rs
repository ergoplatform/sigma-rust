use crate::{
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

pub struct BatchMerkleProof(pub ergo_lib::ergo_merkle_tree::BatchMerkleProof);

pub type BatchMerkleProofPtr = *mut BatchMerkleProof;
pub type ConstBatchMerkleProofPtr = *const BatchMerkleProof;

pub unsafe fn batchmerkleproof_valid(
    proof: ConstBatchMerkleProofPtr,
    expected_root: &[u8],
) -> Result<bool, Error> {
    let proof = const_ptr_as_ref(proof, "proof")?;
    Ok(proof.0.valid(expected_root))
}

pub unsafe fn batchmerkleproof_from_json(
    json: &str,
    proof_out: *mut BatchMerkleProofPtr,
) -> Result<(), Error> {
    let proof_out = mut_ptr_as_mut(proof_out, "proof_out")?;
    *proof_out = Box::into_raw(Box::new(serde_json::from_str(json).map(BatchMerkleProof)?));
    Ok(())
}

pub unsafe fn batchmerkleproof_to_json(proof: ConstBatchMerkleProofPtr) -> Result<String, Error> {
    let proof = const_ptr_as_ref(proof, "proof")?;
    serde_json::to_string(&proof.0).map_err(Error::from)
}
