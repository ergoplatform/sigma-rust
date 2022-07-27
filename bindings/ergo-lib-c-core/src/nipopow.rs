//! Bindings for NiPoPow

use crate::{
    batchmerkleproof::{BatchMerkleProof, BatchMerkleProofPtr},
    block_header::{BlockHeader, BlockHeaderPtr, BlockId, ConstBlockIdPtr},
    collections::{Collection, CollectionPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};
use derive_more::{From, Into};

/// A structure representing NiPoPow proof.
#[derive(Debug, From, Into)]
pub struct NipopowProof(pub(crate) ergo_lib::ergo_nipopow::NipopowProof);
pub type NipopowProofPtr = *mut NipopowProof;
pub type ConstNipopowProofPtr = *const NipopowProof;

#[cfg(feature = "rest")]
impl ergo_lib::ergo_rest::NodeResponse for NipopowProof {}

#[derive(Debug, PartialEq, Eq)]
pub struct PoPowHeader(ergo_lib::ergo_nipopow::PoPowHeader);
pub type PoPowHeaderPtr = *mut PoPowHeader;
pub type ConstPoPowHeaderPtr = *const PoPowHeader;

/// Implementation of the ≥ algorithm from [`KMZ17`], see Algorithm 4
///
/// [`KMZ17`]: https://fc20.ifca.ai/preproceedings/74.pdf
pub unsafe fn nipopow_proof_is_better_than(
    nipopow_proof_ptr: ConstNipopowProofPtr,
    other_nipopow_proof_ptr: ConstNipopowProofPtr,
) -> Result<bool, Error> {
    let self_proof = const_ptr_as_ref(nipopow_proof_ptr, "nipopow_proof_ptr")?;
    let other_proof = const_ptr_as_ref(other_nipopow_proof_ptr, "other_nipopow_proof_ptr")?;
    Ok(self_proof.0.is_better_than(&other_proof.0)?)
}

/// Get suffix head
pub unsafe fn nipopow_proof_suffix_head(
    nipopow_proof_ptr: ConstNipopowProofPtr,
    suffix_head_out: *mut PoPowHeaderPtr,
) -> Result<(), Error> {
    let proof = const_ptr_as_ref(nipopow_proof_ptr, "nipopow_proof_ptr")?;
    let suffix_head_out = mut_ptr_as_mut(suffix_head_out, "suffix_head_out")?;
    *suffix_head_out = Box::into_raw(Box::new(PoPowHeader(proof.0.suffix_head.clone())));
    Ok(())
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
pub struct NipopowVerifier(ergo_lib::ergo_nipopow::NipopowVerifier);
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
        ergo_lib::ergo_nipopow::NipopowVerifier::new(genesis_block_id.0.clone()),
    )));
    Ok(())
}

/// If a best proof exists, allocate a copy and store in `best_proof_out` and return `Ok(true)`.
/// If such a proof doesn't exist return Ok(false).
pub unsafe fn nipopow_verifier_best_proof(
    nipopow_verifier_ptr: ConstNipopowVerifierPtr,
    best_proof_out: *mut NipopowProofPtr,
) -> Result<bool, Error> {
    let verifier = const_ptr_as_ref(nipopow_verifier_ptr, "nipopow_verifier_ptr")?;
    let best_proof_out = mut_ptr_as_mut(best_proof_out, "best_proof_out")?;
    if let Some(proof) = verifier.0.best_proof() {
        *best_proof_out = Box::into_raw(Box::new(NipopowProof(proof)));
        Ok(true)
    } else {
        Ok(false)
    }
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
    verifier.0.process(new_proof.0.clone())?;
    Ok(())
}

pub unsafe fn popow_header_from_json(
    json: &str,
    popow_header_out: *mut PoPowHeaderPtr,
) -> Result<(), Error> {
    let popow_header_out = mut_ptr_as_mut(popow_header_out, "popow_header_out")?;
    let proof = serde_json::from_str(json).map(PoPowHeader)?;
    *popow_header_out = Box::into_raw(Box::new(proof));
    Ok(())
}

pub unsafe fn popow_header_to_json(popow_header_ptr: ConstPoPowHeaderPtr) -> Result<String, Error> {
    let proof = const_ptr_as_ref(popow_header_ptr, "popow_header_ptr")?;
    let s = serde_json::to_string(&proof.0)?;
    Ok(s)
}

pub unsafe fn popow_header_get_interlinks(
    popow_header_ptr: ConstPoPowHeaderPtr,
    interlinks_out: *mut CollectionPtr<BlockId>,
) -> Result<(), Error> {
    let popow_header = const_ptr_as_ref(popow_header_ptr, "popow_header_ptr")?;
    let interlinks_out = mut_ptr_as_mut(interlinks_out, "interlinks_out")?;
    *interlinks_out = Box::into_raw(Box::new(Collection(
        popow_header
            .0
            .interlinks
            .iter()
            .cloned()
            .map(BlockId)
            .collect(),
    )));
    Ok(())
}

pub unsafe fn popow_header_get_header(
    popow_header_ptr: ConstPoPowHeaderPtr,
    header_out: *mut BlockHeaderPtr,
) -> Result<(), Error> {
    let popow_header_ptr = const_ptr_as_ref(popow_header_ptr, "popow_header_ptr")?;
    let header_out = mut_ptr_as_mut(header_out, "header_out")?;
    *header_out = Box::into_raw(Box::new(BlockHeader(popow_header_ptr.0.header.clone())));
    Ok(())
}

pub unsafe fn popow_header_get_interlinks_proof(
    popow_header_ptr: ConstPoPowHeaderPtr,
    proof_out: *mut BatchMerkleProofPtr,
) -> Result<(), Error> {
    let popow_header_ptr = const_ptr_as_ref(popow_header_ptr, "popow_header_ptr")?;
    let proof_out = mut_ptr_as_mut(proof_out, "proof_out")?;
    *proof_out = Box::into_raw(Box::new(BatchMerkleProof(
        popow_header_ptr.0.interlinks_proof.clone(),
    )));
    Ok(())
}

pub unsafe fn popow_header_check_interlinks_proof(
    popow_header_ptr: ConstPoPowHeaderPtr,
) -> Result<bool, Error> {
    let popow_header_ptr = const_ptr_as_ref(popow_header_ptr, "popow_header_ptr")?;
    Ok(popow_header_ptr.0.check_interlinks_proof())
}
