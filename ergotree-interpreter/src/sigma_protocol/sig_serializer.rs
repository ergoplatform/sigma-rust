//! Serialization of proof tree signatures

use super::prover::ProofBytes;
use super::unchecked_tree::UncheckedConjecture;
use super::unchecked_tree::UncheckedLeaf;
use super::unchecked_tree::UncheckedSigmaTree;
use super::unchecked_tree::UncheckedTree;
use crate::sigma_protocol::fiat_shamir::FiatShamirHash;
use crate::sigma_protocol::Challenge;
use crate::sigma_protocol::GroupSizedBytes;
use crate::sigma_protocol::UncheckedSchnorr;

use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use k256::Scalar;
use std::convert::{TryFrom, TryInto};

/// Recursively traverses the given node and serializes challenges and prover messages to the given writer.
/// Note, sigma propositions and commitments are not serialized.
/// Returns the proof bytes containing all the serialized challenges and prover messages (aka `z` values)
pub(crate) fn serialize_sig(tree: UncheckedTree) -> ProofBytes {
    match tree {
        UncheckedTree::NoProof => ProofBytes::Empty,
        UncheckedTree::UncheckedSigmaTree(ust) => {
            let mut data = Vec::new();
            let mut w = SigmaByteWriter::new(&mut data, None);
            sig_write_bytes(&ust, &mut w, true)
                // since serialization may fail only for underlying IO errors it's ok to force unwrap
                .expect("serialization failed");
            ProofBytes::Some(data)
        }
    }
}

/// Recursively traverses the given node and serializes challenges and prover messages to the given writer.
/// Note, sigma propositions and commitments are not serialized.
/// Returns the proof bytes containing all the serialized challenges and prover messages (aka `z` values)
fn sig_write_bytes<W: SigmaByteWrite>(
    node: &UncheckedSigmaTree,
    w: &mut W,
    write_challenges: bool,
) -> Result<(), std::io::Error> {
    if write_challenges {
        node.challenge().sigma_serialize(w)?;
    }
    match node {
        UncheckedSigmaTree::UncheckedLeaf(leaf) => match leaf {
            UncheckedLeaf::UncheckedSchnorr(us) => {
                let mut sm_bytes = us.second_message.z.to_bytes();
                w.write_all(sm_bytes.as_mut_slice())
            }
        },
        UncheckedSigmaTree::UncheckedConjecture(conj) => match conj {
            UncheckedConjecture::CandUnchecked {
                challenge: _,
                children,
            } => {
                // don't write children's challenges -- they are equal to the challenge of this node
                for child in children {
                    sig_write_bytes(child, w, false)?;
                }
                Ok(())
            }
        },
    }
}

/// Verifier Step 2: In a top-down traversal of the tree, obtain the challenges for the children of every
/// non-leaf node by reading them from the proof or computing them.
/// Verifier Step 3: For every leaf node, read the response z provided in the proof.
pub(crate) fn parse_sig_compute_challenges(
    exp: SigmaBoolean,
    proof: &ProofBytes,
) -> Result<UncheckedTree, SigParsingError> {
    if let ProofBytes::Some(proof_bytes) = proof {
        // Verifier Step 2: Let e_0 be the challenge in the node here (e_0 is called "challenge" in the code)
        let chal_len = super::SOUNDNESS_BYTES;
        let challenge = if let Some(bytes) = proof_bytes.get(..chal_len) {
            // safe since it should only be of the required size
            Challenge::from(FiatShamirHash::try_from(bytes).unwrap())
        } else {
            return Err(SigParsingError::InvalidProofSize);
        };
        match exp {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(dl)) => {
                let scalar_bytes: &[u8; super::GROUP_SIZE] =
                    match proof_bytes.get(chal_len..chal_len + super::GROUP_SIZE) {
                        Some(v) => v.try_into().unwrap(), // safe, since it should only be of this size
                        None => return Err(SigParsingError::InvalidProofSize),
                    };
                let z = Scalar::from(GroupSizedBytes::from(scalar_bytes));
                Ok(UncheckedSchnorr {
                    proposition: dl,
                    commitment_opt: None,
                    challenge,
                    second_message: z.into(),
                }
                .into())
            }
            _ => todo!(),
        }
    } else {
        Err(SigParsingError::InvalidProofSize)
    }
}

/// Errors when parsing proof tree signatures
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum SigParsingError {
    /// Invalid proof size (expected 32 bytes)
    InvalidProofSize,
}
