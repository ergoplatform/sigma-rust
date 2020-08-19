use super::{
    sigma_boolean::SigmaProp,
    unchecked_tree::{UncheckedSigmaTree, UncheckedTree},
    unproven_tree::UnprovenTree,
    ProofTree, ProofTreeLeaf, ProverMessage, SOUNDNESS_BYTES,
};
use crate::{ast::Expr, serialization::SigmaSerializable, ErgoTree};
use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;
use std::{
    convert::{TryFrom, TryInto},
    rc::Rc,
};
use thiserror::Error;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FiatShamirHash(pub Box<[u8; SOUNDNESS_BYTES]>);

pub fn fiat_shamir_hash_fn(input: &[u8]) -> FiatShamirHash {
    // unwrap is safe 24 bytes is a valid hash size (<= 512 && 24 % 8 == 0)
    let mut hasher = VarBlake2b::new(SOUNDNESS_BYTES).unwrap();
    hasher.update(input);
    let hash = hasher.finalize_boxed();
    // unwrap is safe due to hash size is expected to be 24
    FiatShamirHash(hash.try_into().unwrap())
}

impl Into<[u8; SOUNDNESS_BYTES]> for FiatShamirHash {
    fn into(self) -> [u8; SOUNDNESS_BYTES] {
        *self.0
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
pub fn fiat_shamir_tree_to_bytes(tree: &ProofTree) -> Vec<u8> {
    const LEAF_PREFIX: u8 = 1;

    let leaf: &dyn ProofTreeLeaf = match tree {
        ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(
            UncheckedSigmaTree::UncheckedLeaf(ul),
        )) => ul,
        ProofTree::UnprovenTree(UnprovenTree::UnprovenLeaf(ul)) => ul,
        _ => todo!(),
    };

    let prop_tree = ErgoTree::with_segregation(Rc::new(Expr::Const(
        SigmaProp::new(leaf.proposition()).into(),
    )));
    let mut prop_bytes = prop_tree.sigma_serialise_bytes();
    // TODO: is unwrap safe here?
    let mut commitment_bytes = leaf.commitment_opt().unwrap().bytes();
    let mut res = vec![LEAF_PREFIX];
    res.append((prop_bytes.len() as u16).to_be_bytes().to_vec().as_mut());
    res.append(prop_bytes.as_mut());
    res.append(
        (commitment_bytes.len() as u16)
            .to_be_bytes()
            .to_vec()
            .as_mut(),
    );
    res.append(commitment_bytes.as_mut());
    res
}
