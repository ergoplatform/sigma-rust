//! Serialization of proof tree signatures

use std::convert::TryInto;

use super::gf2_192::gf2_192poly_from_byte_array;
use super::prover::ProofBytes;
use super::unchecked_tree::UncheckedConjecture;
use super::unchecked_tree::UncheckedLeaf;
use super::unchecked_tree::UncheckedTree;
use super::GROUP_SIZE;
use super::SOUNDNESS_BYTES;
use crate::sigma_protocol::dht_protocol::SecondDhTupleProverMessage;
use crate::sigma_protocol::unchecked_tree::UncheckedDhTuple;
use crate::sigma_protocol::Challenge;
use crate::sigma_protocol::GroupSizedBytes;
use crate::sigma_protocol::UncheckedSchnorr;

use ergotree_ir::serialization::sigma_byte_reader;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;

use derive_more::From;
use gf2_192::Gf2_192Error;
use k256::Scalar;
use thiserror::Error;

/// Recursively traverses the given node and serializes challenges and prover messages to the given writer.
/// Note, sigma propositions and commitments are not serialized.
/// Returns the proof bytes containing all the serialized challenges and prover messages (aka `z` values)
pub(crate) fn serialize_sig(tree: UncheckedTree) -> ProofBytes {
    let mut data = Vec::with_capacity(SOUNDNESS_BYTES + GROUP_SIZE);
    let mut w = SigmaByteWriter::new(&mut data, None);
    #[allow(clippy::unwrap_used)]
    // since serialization may fail only for underlying IO errors (OOM, etc.) it's ok to force unwrap
    sig_write_bytes(&tree, &mut w, true).unwrap();
    ProofBytes::Some(data)
}

/// Recursively traverses the given node and serializes challenges and prover messages to the given writer.
/// Note, sigma propositions and commitments are not serialized.
/// Returns the proof bytes containing all the serialized challenges and prover messages (aka `z` values)
fn sig_write_bytes<W: SigmaByteWrite>(
    node: &UncheckedTree,
    w: &mut W,
    write_challenges: bool,
) -> Result<(), std::io::Error> {
    if write_challenges {
        node.challenge().sigma_serialize(w)?;
    }
    match node {
        UncheckedTree::UncheckedLeaf(leaf) => match leaf {
            UncheckedLeaf::UncheckedSchnorr(us) => {
                let mut sm_bytes = us.second_message.z.to_bytes();
                w.write_all(sm_bytes.as_mut_slice())?;
                Ok(())
            }
            UncheckedLeaf::UncheckedDhTuple(dh) => {
                let mut sm_bytes = dh.second_message.z.to_bytes();
                w.write_all(sm_bytes.as_mut_slice())
            }
        },
        UncheckedTree::UncheckedConjecture(conj) => match conj {
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
            UncheckedConjecture::CorUnchecked {
                challenge: _,
                children,
            } => {
                // don't write last child's challenge -- it's computed by the verifier via XOR
                let (last, elements) = children.split_last();
                for child in elements {
                    sig_write_bytes(child, w, true)?;
                }
                sig_write_bytes(last, w, false)?;
                Ok(())
            }
            UncheckedConjecture::CthresholdUnchecked {
                challenge: _,
                children,
                k: _,
                polynomial,
            } => {
                // write the polynomial, except the zero-degree coefficient
                w.write_all(polynomial.to_bytes().as_mut_slice())?;
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
    mut proof_bytes: Vec<u8>,
) -> Result<UncheckedTree, SigParsingError> {
    let mut r = sigma_byte_reader::from_bytes(proof_bytes.as_mut_slice());
    parse_sig_compute_challenges_reader(exp, &mut r, None)
}

/// Verifier Step 2: In a top-down traversal of the tree, obtain the challenges for the children of every
/// non-leaf node by reading them from the proof or computing them.
/// Verifier Step 3: For every leaf node, read the response z provided in the proof.
/// * `exp` - sigma proposition which defines the structure of bytes from the reader
fn parse_sig_compute_challenges_reader<R: SigmaByteRead>(
    exp: &SigmaBoolean,
    r: &mut R,
    challenge_opt: Option<Challenge>,
) -> Result<UncheckedTree, SigParsingError> {
    // Verifier Step 2: Let e_0 be the challenge in the node here (e_0 is called "challenge" in the code)
    let challenge = if let Some(c) = challenge_opt {
        c
    } else {
        Challenge::sigma_parse(r)?
    };

    match exp {
        SigmaBoolean::TrivialProp(_) => Err(SigParsingError::Unexpected(
            "parse_sig_compute_challenges: TrivialProp should be handled before this call",
        )),
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
            SigmaProofOfKnowledgeTree::ProveDhTuple(dh) => {
                // Verifier Step 3: For every leaf node, read the response z provided in the proof.
                let mut scalar_bytes: [u8; super::GROUP_SIZE] = [0; super::GROUP_SIZE];
                r.read_exact(&mut scalar_bytes)?;
                let z = Scalar::from(GroupSizedBytes(scalar_bytes.into()));
                Ok(UncheckedDhTuple {
                    proposition: dh.clone(),
                    commitment_opt: None,
                    challenge,
                    second_message: SecondDhTupleProverMessage { z },
                }
                .into())
            }
        },
        SigmaBoolean::SigmaConjecture(conj) => match conj {
            SigmaConjecture::Cand(cand) => {
                // Verifier Step 2: If the node is AND, then all of its children get e_0 as
                // the challenge
                let children = cand.items.try_mapped_ref(|it| {
                    parse_sig_compute_challenges_reader(it, r, Some(challenge.clone()))
                })?;
                Ok(UncheckedConjecture::CandUnchecked {
                    challenge,
                    children,
                }
                .into())
            }
            SigmaConjecture::Cor(cor) => {
                // Verifier Step 2: If the node is OR, then each of its children except rightmost
                // one gets the challenge given in the proof for that node.
                // The rightmost child gets a challenge computed as an XOR of the challenges of all the other children and e_0.

                // Read all the children but the last and compute the XOR of all the challenges including e_0
                let mut children: Vec<UncheckedTree> = Vec::with_capacity(cor.items.len());

                let (last, rest) = cor.items.split_last();
                for it in rest {
                    children.push(parse_sig_compute_challenges_reader(it, r, None)?);
                }
                let xored_challenge = children
                    .clone()
                    .into_iter()
                    .map(|c| c.challenge())
                    .fold(challenge.clone(), |acc, c| acc.xor(c));
                let last_child =
                    parse_sig_compute_challenges_reader(last, r, Some(xored_challenge))?;
                children.push(last_child);

                #[allow(clippy::unwrap_used)] // since quantity is preserved unwrap is safe here
                Ok(UncheckedConjecture::CorUnchecked {
                    challenge,
                    children: children.try_into().unwrap(),
                }
                .into())
            }
            SigmaConjecture::Cthreshold(ct) => {
                // Verifier Step 2: If the node is THRESHOLD,
                // evaluate the polynomial Q(x) at points 1, 2, ..., n to get challenges for child 1, 2, ..., n, respectively.
                // Read the polynomial -- it has n-k coefficients
                let n_children = ct.children.len();
                let n_coeff = n_children - ct.k as usize;
                let buf_size = n_coeff * SOUNDNESS_BYTES;
                let mut coeff_bytes = vec![0u8; buf_size];
                r.read_exact(&mut coeff_bytes)?;
                let polynomial = gf2_192poly_from_byte_array(challenge.clone(), coeff_bytes)?;

                let children =
                    ct.children
                        .clone()
                        .enumerated()
                        .try_mapped_ref(|(idx, child)| {
                            // Note the cast to `u8` is safe since `ct.children` is of type
                            // `SigmaConjectureItems<_>` which is a `BoundedVec<_, 2, 255>`.
                            let one_based_index = (idx + 1) as u8;
                            let ch = polynomial.evaluate(one_based_index).into();
                            parse_sig_compute_challenges_reader(child, r, Some(ch))
                        })?;
                Ok(UncheckedConjecture::CthresholdUnchecked {
                    challenge,
                    children,
                    k: ct.k,
                    polynomial,
                }
                .into())
            }
        },
    }
}

/// Errors when parsing proof tree signatures
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum SigParsingError {
    /// `gf2_192` error
    #[error("gf2_192 error: {0}")]
    Gf2_192Error(Gf2_192Error),
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(SigmaParsingError),
    /// Unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(&'static str),
}

impl From<std::io::Error> for SigParsingError {
    fn from(e: std::io::Error) -> Self {
        SigParsingError::IoError(e.to_string())
    }
}
