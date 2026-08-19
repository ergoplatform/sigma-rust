use ergo_chain_types::{autolykos_pow_scheme::AutolykosPowSchemeError, BlockId, Header};

use crate::nipopow_proof::{NipopowProof, NipopowProofError};

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

    /// Returns best proof
    pub fn best_proof(&self) -> Option<NipopowProof> {
        self.best_proof.clone()
    }

    /// Returns chain of `Header`s from the best proof.
    pub fn best_chain(&self) -> Vec<Header> {
        self.best_proof
            .as_ref()
            .map_or_else(Vec::new, |p| p.headers_chain().cloned().collect())
    }

    /// Process given proof
    pub fn process(&mut self, new_proof: NipopowProof) -> Result<(), NipopowProofError> {
        let h = new_proof.headers_chain().next();
        if let Some(h) = h {
            if h.id == self.genesis_block_id {
                if let Some(p) = &self.best_proof {
                    // Direct comparison intentionally returns `Ok(false)` for
                    // different proof parameters. At the stateful verifier
                    // boundary, keep that incompatibility observable instead
                    // of making it indistinguishable from an ordinary loss.
                    if new_proof.is_valid() && (new_proof.m, new_proof.k) != (p.m, p.k) {
                        return Err(NipopowProofError::AutolykosPowSchemeError(
                            AutolykosPowSchemeError::OutOfBounds,
                        ));
                    }
                    if new_proof.is_better_than(p)? {
                        self.best_proof = Some(new_proof);
                    }
                } else if new_proof.is_valid() {
                    self.best_proof = Some(new_proof);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::nipopow_proof::tests::valid_proof;
    use ergo_chain_types::{autolykos_pow_scheme::AutolykosPowSchemeError, Digest32};

    #[test]
    fn process_reports_valid_parameter_mismatch_but_ignores_invalid_challenger() {
        let incumbent = valid_proof(6, 2);
        let genesis_id = incumbent.suffix_head.header.id;
        let mut verifier = NipopowVerifier::new(genesis_id);
        verifier.process(incumbent.clone()).unwrap();

        assert_eq!(
            verifier.process(valid_proof(7, 2)),
            Err(NipopowProofError::AutolykosPowSchemeError(
                AutolykosPowSchemeError::OutOfBounds
            ))
        );
        assert_eq!(verifier.best_proof(), Some(incumbent.clone()));

        let mut invalid = valid_proof(7, 2);
        invalid.suffix_tail[0].parent_id = BlockId(Digest32::from([0xff; 32]));
        assert_eq!(verifier.process(invalid), Ok(()));
        assert_eq!(verifier.best_proof(), Some(incumbent));
    }
}
