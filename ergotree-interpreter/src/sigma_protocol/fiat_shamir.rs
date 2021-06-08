//! Fiat-Shamir transformation

use super::crypto_utils::secure_random_bytes;
use super::proof_tree::ProofTreeKind;
use crate::sigma_protocol::ProverMessage;
use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;
use ergotree_ir::ergo_tree::{ErgoTree, ErgoTreeHeader};
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
use std::convert::{TryFrom, TryInto};
use thiserror::Error;

#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;

use super::proof_tree::ProofTree;
use super::GROUP_SIZE;
use super::SOUNDNESS_BYTES;

/// Hash type for Fiat-Shamir hash function (24-bytes)
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FiatShamirHash(pub Box<[u8; SOUNDNESS_BYTES]>);

impl FiatShamirHash {
    pub fn secure_random() -> Self {
        #[allow(clippy::unwrap_used)] // since we set the correct size
        secure_random_bytes(SOUNDNESS_BYTES)
            .as_slice()
            .try_into()
            .unwrap()
    }
}

/// Fiat-Shamir hash function
pub fn fiat_shamir_hash_fn(input: &[u8]) -> FiatShamirHash {
    // unwrap is safe, since 32 bytes is a valid hash size (<= 512 && 24 % 8 == 0)
    #[allow(clippy::unwrap_used)]
    let mut hasher = VarBlake2b::new(GROUP_SIZE).unwrap();
    hasher.update(input);
    let hash = hasher.finalize_boxed();
    let taken: Vec<u8> = hash.to_vec().into_iter().take(SOUNDNESS_BYTES).collect();
    // unwrap is safe due to hash size is expected to be SOUNDNESS_BYTES
    #[allow(clippy::unwrap_used)]
    FiatShamirHash(taken.into_boxed_slice().try_into().unwrap())
}

impl From<FiatShamirHash> for [u8; SOUNDNESS_BYTES] {
    fn from(v: FiatShamirHash) -> Self {
        *v.0
    }
}

impl TryFrom<&[u8]> for FiatShamirHash {
    type Error = FiatShamirHashError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let arr: [u8; SOUNDNESS_BYTES] = value.try_into()?;
        Ok(FiatShamirHash(Box::new(arr)))
    }
}

/// Invalid byte array size
#[derive(Error, Debug)]
#[error("Invalid byte array size ({0})")]
pub struct FiatShamirHashError(std::array::TryFromSliceError);

impl From<std::array::TryFromSliceError> for FiatShamirHashError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        FiatShamirHashError(err)
    }
}

///  Prover Step 7: Convert the tree to a string s for input to the Fiat-Shamir hash function.
///  The conversion should be such that the tree can be unambiguously parsed and restored given the string.
///  For each non-leaf node, the string should contain its type (OR or AND).
///  For each leaf node, the string should contain the Sigma-protocol statement being proven and the commitment.
///  The string should not contain information on whether a node is marked "real" or "simulated",
///  and should not contain challenges, responses, or the real/simulated flag for any node.
pub(crate) fn fiat_shamir_tree_to_bytes(
    tree: &ProofTree,
) -> Result<Vec<u8>, FiatShamirTreeSerializationError> {
    let mut data = Vec::new();
    let mut w = SigmaByteWriter::new(&mut data, None);
    fiat_shamir_write_bytes(tree, &mut w)?;
    Ok(data)
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("FiatShamirTreeSerializationError: {0}")]
pub struct FiatShamirTreeSerializationError(String);

impl From<std::io::Error> for FiatShamirTreeSerializationError {
    fn from(error: std::io::Error) -> Self {
        FiatShamirTreeSerializationError(error.to_string())
    }
}

fn fiat_shamir_write_bytes<W: SigmaByteWrite>(
    tree: &ProofTree,
    w: &mut W,
) -> Result<(), FiatShamirTreeSerializationError> {
    const INTERNAL_NODE_PREFIX: u8 = 0;
    const LEAF_PREFIX: u8 = 1;

    Ok(match tree.as_tree_kind() {
        ProofTreeKind::Leaf(leaf) => {
            let prop_tree = ErgoTree::new(
                ErgoTreeHeader::v0(true),
                &Expr::Const(SigmaProp::new(leaf.proposition()).into()),
            );
            let prop_bytes = prop_tree.sigma_serialize_bytes();
            let commitment_bytes = leaf
                .commitment_opt()
                .ok_or_else(|| {
                    FiatShamirTreeSerializationError(format!("empty commitment in {:?}", leaf))
                })?
                .bytes();
            w.put_u8(LEAF_PREFIX)?;
            w.put_i16_be_bytes(prop_bytes.len() as i16)?;
            w.write_all(prop_bytes.as_ref())?;
            w.put_i16_be_bytes(commitment_bytes.len() as i16)?;
            w.write_all(commitment_bytes.as_slice())?
        }
        ProofTreeKind::Conjecture(c) => {
            w.put_u8(INTERNAL_NODE_PREFIX)?;
            w.put_u8(c.conjecture_type() as u8)?;
            w.put_i16_be_bytes(c.children().len() as i16)?;
            for child in c.children().iter() {
                fiat_shamir_write_bytes(&child, w)?;
            }
        }
    })
}
