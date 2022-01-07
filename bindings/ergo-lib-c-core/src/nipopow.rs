//! Bindings for NiPoPow

use crate::{
    block_header::{BlockHeader, ConstBlockIdPtr},
    collections::{Collection, CollectionPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// A structure representing NiPoPow proof.
#[derive(Debug)]
pub struct NipopowProof(ergo_lib::ergo_nipopow::nipopow_proof::NipopowProof);
pub type NipopowProofPtr = *mut NipopowProof;
pub type ConstNipopowProofPtr = *const NipopowProof;

/// Implementation of the ≥ algorithm from [`KMZ17`], see Algorithm 4
///
/// [`KMZ17`]: https://fc20.ifca.ai/preproceedings/74.pdf
pub unsafe fn nipopow_proof_is_better_than(
    nipopow_proof_ptr: ConstNipopowProofPtr,
    other_nipopow_proof_ptr: ConstNipopowProofPtr,
) -> Result<bool, Error> {
    let self_proof = const_ptr_as_ref(nipopow_proof_ptr, "nipopow_proof_ptr")?;
    let other_proof = const_ptr_as_ref(other_nipopow_proof_ptr, "other_nipopow_proof_ptr")?;
    Ok(self_proof.0.is_better_than(&other_proof.0))
}

/// Parse from JSON.
pub unsafe fn nipopow_proof_from_json(
    json: &str,
    nipopow_proof_out: *mut NipopowProofPtr,
) -> Result<(), Error> {
    let nipopow_proof_out = mut_ptr_as_mut(nipopow_proof_out, "nipopow_proof_out")?;
    let proof = serde_json::from_str(json).map(NipopowProof)?;
    *nipopow_proof_out = Box::into_raw(Box::new(proof));
    Ok(())
}

/// JSON representation as text
pub unsafe fn nipopow_proof_to_json(
    nipopow_proof_ptr: ConstNipopowProofPtr,
) -> Result<String, Error> {
    let proof = const_ptr_as_ref(nipopow_proof_ptr, "nipopow_proof_ptr")?;
    let s = serde_json::to_string(&proof.0)?;
    Ok(s)
}

/// A verifier for PoPoW proofs. During its lifetime, it processes many proofs with the aim of
/// deducing at any given point what is the best (sub)chain rooted at the specified genesis.
#[derive(Debug)]
pub struct NipopowVerifier(ergo_lib::ergo_nipopow::nipopow_verifier::NipopowVerifier);
pub type NipopowVerifierPtr = *mut NipopowVerifier;
pub type ConstNipopowVerifierPtr = *const NipopowVerifier;

/// Create new instance
pub unsafe fn nipopow_verifier_new(
    genesis_block_id_ptr: ConstBlockIdPtr,
    nipopow_verifier_out: *mut NipopowVerifierPtr,
) -> Result<(), Error> {
    let genesis_block_id = const_ptr_as_ref(genesis_block_id_ptr, "genesis_block_id_ptr")?;
    let nipopow_verifier_out = mut_ptr_as_mut(nipopow_verifier_out, "nipopow_verifier_out")?;
    *nipopow_verifier_out = Box::into_raw(Box::new(NipopowVerifier(
        ergo_lib::ergo_nipopow::nipopow_verifier::NipopowVerifier::new(genesis_block_id.0.clone()),
    )));
    Ok(())
}

/// Returns chain of `BlockHeader`s from the best proof.
pub unsafe fn nipopow_verifier_best_chain(
    nipopow_verifier_ptr: ConstNipopowVerifierPtr,
    best_chain_out: *mut CollectionPtr<BlockHeader>,
) -> Result<(), Error> {
    let verifier = const_ptr_as_ref(nipopow_verifier_ptr, "nipopow_verifier_ptr")?;
    let best_chain_out = mut_ptr_as_mut(best_chain_out, "best_chain_out")?;
    *best_chain_out = Box::into_raw(Box::new(Collection(
        verifier
            .0
            .best_chain()
            .into_iter()
            .map(BlockHeader)
            .collect(),
    )));
    Ok(())
}

/// Process given proof
pub unsafe fn nipopow_verifier_process(
    nipopow_verifier_ptr: NipopowVerifierPtr,
    nipopow_proof_ptr: ConstNipopowProofPtr,
) -> Result<(), Error> {
    let verifier = mut_ptr_as_mut(nipopow_verifier_ptr, "nipopow_verifier_ptr")?;
    let new_proof = const_ptr_as_ref(nipopow_proof_ptr, "nipopow_proof_ptr")?;
    verifier.0.process(new_proof.0.clone());
    Ok(())
}
