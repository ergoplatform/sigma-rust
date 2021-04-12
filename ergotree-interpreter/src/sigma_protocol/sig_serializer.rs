//! Serialization of proof tree signatures

use super::prover::ProofBytes;
use super::unchecked_tree::UncheckedConjecture;
use super::unchecked_tree::UncheckedLeaf;
use super::unchecked_tree::UncheckedSigmaTree;
use super::unchecked_tree::UncheckedTree;
use crate::sigma_protocol::fiat_shamir::FiatShamirHash;
use crate::sigma_protocol::unchecked_tree::UncheckedConjecture::CandUnchecked;
use crate::sigma_protocol::Challenge;
use crate::sigma_protocol::GroupSizedBytes;
use crate::sigma_protocol::UncheckedSchnorr;

use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use k256::Scalar;
use std::io::Read;

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
/// * `exp` - sigma proposition which defines the structure of bytes from the reader
/// * `proof` - proof to extract challenges from
pub(crate) fn parse_sig_compute_challenges(
    exp: &SigmaBoolean,
    proof: ProofBytes,
) -> Result<UncheckedTree, SigParsingError> {
    match proof {
        ProofBytes::Empty => Ok(UncheckedTree::NoProof),
        ProofBytes::Some(mut proof_bytes) => {
            let mut r = std::io::Cursor::new(proof_bytes.as_mut_slice());
            parse_sig_compute_challnges_reader(exp, &mut r, None).map(|tree| tree.into())
        }
    }
}

/// Verifier Step 2: In a top-down traversal of the tree, obtain the challenges for the children of every
/// non-leaf node by reading them from the proof or computing them.
/// Verifier Step 3: For every leaf node, read the response z provided in the proof.
/// * `exp` - sigma proposition which defines the structure of bytes from the reader
fn parse_sig_compute_challnges_reader<R: Read>(
    exp: &SigmaBoolean,
    r: &mut R,
    challenge_opt: Option<Challenge>,
) -> Result<UncheckedSigmaTree, SigParsingError> {
    // Verifier Step 2: Let e_0 be the challenge in the node here (e_0 is called "challenge" in the code)
    let challenge = if let Some(c) = challenge_opt {
        c
    } else {
        let mut chal_bytes: [u8; super::SOUNDNESS_BYTES] = [0; super::SOUNDNESS_BYTES];
        r.read_exact(&mut chal_bytes)?;
        Challenge::from(FiatShamirHash(Box::new(chal_bytes)))
    };

    match exp {
        SigmaBoolean::TrivialProp(_) => {
            panic!("TrivialProp should be handled before this call")
        }
        SigmaBoolean::ProofOfKnowledge(tree) => match tree {
            SigmaProofOfKnowledgeTree::ProveDlog(dl) => {
                // Verifier Step 3: For every leaf node, read the response z provided in the proof.
                let mut scalar_bytes: [u8; super::GROUP_SIZE] = [0; super::GROUP_SIZE];
                r.read_exact(&mut scalar_bytes)?;
                let z = Scalar::from(GroupSizedBytes(scalar_bytes.into()));
                Ok(UncheckedSchnorr {
                    proposition: dl.clone(),
                    commitment_opt: None,
                    challenge,
                    second_message: z.into(),
                }
                .into())
            }
            SigmaProofOfKnowledgeTree::ProveDhTuple(_) => todo!("DHT is not yet supported"),
        },
        SigmaBoolean::SigmaConjecture(conj) => match conj {
            SigmaConjecture::Cand(cand) => {
                // Verifier Step 2: If the node is AND, then all of its children get e_0 as
                // the challenge
                let mut children: Vec<UncheckedSigmaTree> = Vec::new();
                for it in cand.items.clone() {
                    children.push(parse_sig_compute_challnges_reader(
                        &it,
                        r,
                        Some(challenge.clone()),
                    )?);
                }
                Ok(CandUnchecked {
                    challenge,
                    children,
                }
                .into())
            }
            SigmaConjecture::Cor(_) => todo!("OR is not yet supported"),
        },
    }
}

// TODO: use io::Error directly?
/// Errors when parsing proof tree signatures
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SigParsingError {
    /// IO error
    IoError(String),
}

impl From<std::io::Error> for SigParsingError {
    fn from(e: std::io::Error) -> Self {
        SigParsingError::IoError(e.to_string())
    }
}
