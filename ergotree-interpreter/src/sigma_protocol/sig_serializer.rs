//! Serialization of proof tree signatures

use std::convert::TryInto;

use super::gf2_192::gf2_192poly_from_byte_array;
use super::prover::ProofBytes;
use super::unchecked_tree::UncheckedConjecture;
use super::unchecked_tree::UncheckedLeaf;
use super::unchecked_tree::UncheckedTree;
use super::wscalar::Wscalar;
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
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;

use gf2_192::Gf2_192Error;
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
                let mut sm_bytes = us.second_message.z.as_scalar_ref().to_bytes();
                w.write_all(sm_bytes.as_mut_slice())?;
                Ok(())
            }
            UncheckedLeaf::UncheckedDhTuple(dh) => {
                let mut sm_bytes = dh.second_message.z.as_scalar_ref().to_bytes();
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
                k,
                polynomial,
            } => {
                let mut polynomial_bytes = polynomial.to_bytes();
                assert_eq!(
                    polynomial_bytes.len(),
                    (children.len() - *k as usize) * SOUNDNESS_BYTES
                );
                // write the polynomial, except the zero-degree coefficient
                w.write_all(polynomial_bytes.as_mut_slice())?;
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
pub fn parse_sig_compute_challenges(
    exp: &SigmaBoolean,
    mut proof_bytes: Vec<u8>,
) -> Result<UncheckedTree, SigParsingError> {
    if proof_bytes.is_empty() {
        return Err(SigParsingError::EmptyProof(exp.clone()));
    }
    let mut r = sigma_byte_reader::from_bytes(proof_bytes.as_mut_slice());
    parse_sig_compute_challenges_reader(exp, &mut r, None)
        .map_err(|e| SigParsingError::TopLevelExpWrap(e.into(), exp.clone()))
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
        Challenge::sigma_parse(r).map_err(|_| SigParsingError::ChallengeRead(exp.clone()))?
    };

    match exp {
        SigmaBoolean::TrivialProp(b) => Err(SigParsingError::TrivialPropFound(*b)),
        SigmaBoolean::ProofOfKnowledge(tree) => match tree {
            SigmaProofOfKnowledgeTree::ProveDlog(dl) => {
                // Verifier Step 3: For every leaf node, read the response z provided in the proof.
                let z = read_scalar(r)
                    .map_err(|_| SigParsingError::ScalarReadProveDlog(exp.clone()))?;
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
                let z = read_scalar(r)
                    .map_err(|_| SigParsingError::ScalarReadProveDhTuple(exp.clone()))?;
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
                r.read_exact(&mut coeff_bytes)
                    .map_err(|_| SigParsingError::CthresholdCoeffRead(exp.clone()))?;
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

fn read_scalar<R: SigmaByteRead>(r: &mut R) -> Result<Wscalar, SigmaSerializationError> {
    let mut scalar_bytes: Box<[u8; super::GROUP_SIZE]> = Box::new([0; super::GROUP_SIZE]);
    let bytes_read = r.read(&mut *scalar_bytes)?;
    scalar_bytes.rotate_right(super::GROUP_SIZE - bytes_read);
    Ok(Wscalar::from(GroupSizedBytes(scalar_bytes)))
}

/// Errors when parsing proof tree signatures
#[allow(missing_docs)]
#[derive(Error, PartialEq, Debug, Clone)]
pub enum SigParsingError {
    #[error("Empty proof for exp: {0:?}")]
    EmptyProof(SigmaBoolean),

    #[error("Unexpected TrivialProp found: {0}")]
    TrivialPropFound(bool),

    #[error("gf2_192 error: {0}")]
    Gf2_192Error(#[from] Gf2_192Error),

    #[error("Empty commitment in UncheckedLeaf with proposition: {0:?}")]
    EmptyCommitment(SigmaBoolean),

    #[error("Challenge reading erorr with exp: {0:?}")]
    ChallengeRead(SigmaBoolean),

    #[error("Scalar in ProveDlog reading erorr with exp: {0:?}")]
    ScalarReadProveDlog(SigmaBoolean),

    #[error("Scalar in ProveDhTumple reading erorr with exp: {0:?}")]
    ScalarReadProveDhTuple(SigmaBoolean),

    #[error("Cthreshold coeff reading erorr with exp: {0:?}")]
    CthresholdCoeffRead(SigmaBoolean),

    #[error("Error: {0:?} for top level exp: {1:?}")]
    TopLevelExpWrap(Box<SigParsingError>, SigmaBoolean),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use std::io::Cursor;

    use ergotree_ir::serialization::{
        constant_store::ConstantStore, sigma_byte_reader::SigmaByteReader,
    };
    use k256::Scalar;

    use super::read_scalar;

    // Test scalar parsing and also test handling parsing when there are less than GROUP_SIZE bytes in the buffer
    #[test]
    fn test_scalar_parse() {
        let mut bytes = [0; 32];
        bytes[31] = 1;

        for i in 0..31 {
            let cursor = Cursor::new(&bytes[i..]);
            let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
            let scalar = read_scalar(&mut sr).unwrap();
            assert_eq!(*scalar.as_scalar_ref(), Scalar::ONE);
        }
    }
}
