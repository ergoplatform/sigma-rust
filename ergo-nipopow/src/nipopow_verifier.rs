use ergotree_ir::chain::{block_id::BlockId, header::Header};

use crate::nipopow_proof::NipopowProof;

/// A verifier for PoPoW proofs. During its lifetime, it processes many proofs with the aim of
/// deducing at any given point what is the best (sub)chain rooted at the specified genesis.
#[derive(Debug)]
pub struct NipopowVerifier {
    best_proof: Option<NipopowProof>,
    /// `BlockId` of the genesis block.
    genesis_block_id: BlockId,
}

impl NipopowVerifier {
    /// Create new instance
    pub fn new(genesis_block_id: BlockId) -> Self {
        NipopowVerifier {
            best_proof: None,
            genesis_block_id,
        }
    }

    /// Returns chain of `Header`s from the best proof.
    pub fn best_chain(&self) -> Vec<Header> {
        self.best_proof
            .as_ref()
            .map_or_else(Vec::new, |p| p.headers_chain().cloned().collect())
    }

    /// Process given proof
    pub fn process(&mut self, new_proof: NipopowProof) {
        let h = new_proof.headers_chain().next();
        if let Some(h) = h {
            if h.id == self.genesis_block_id {
                if let Some(p) = &self.best_proof {
                    if new_proof.is_better_than(p) {
                        self.best_proof = Some(new_proof);
                    }
                }
            }
        }
    }
}
