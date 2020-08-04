//! Sigma protocols

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(missing_docs)]

pub mod dlog_group;
pub mod dlog_protocol;
pub mod prover;
pub mod verifier;

use k256::arithmetic::Scalar;

use crate::{big_integer::BigInteger, serialization::op_code::OpCode};
use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;
use dlog_group::EcPoint;
use dlog_protocol::{FirstDlogProverMessage, SecondDlogProverMessage};
use std::convert::TryInto;

/// Secret key of discrete logarithm signature protocol
pub struct DlogProverInput {
    /// secret key value
    pub w: Scalar,
}

impl DlogProverInput {
    /// generates random secret in the range [0, n), where n is DLog group order.
    pub fn random() -> DlogProverInput {
        DlogProverInput {
            w: dlog_group::random_scalar_in_group_range(),
        }
    }

    /// public key of discrete logarithm signature protocol
    fn public_image(&self) -> ProveDlog {
        // test it, see https://github.com/ergoplatform/sigma-rust/issues/38
        let g = dlog_group::generator();
        ProveDlog::new(dlog_group::exponentiate(&g, &self.w))
    }
}

/// Private inputs (secrets)
pub enum PrivateInput {
    DlogProverInput(DlogProverInput),
    DiffieHellmanTupleProverInput,
}

/// Construct a new SigmaBoolean value representing public key of discrete logarithm signature protocol.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ProveDlog {
    /// public key
    pub h: Box<EcPoint>,
}

impl ProveDlog {
    /// create new public key
    pub fn new(ecpoint: EcPoint) -> ProveDlog {
        ProveDlog {
            h: Box::new(ecpoint),
        }
    }
}

/// Construct a new SigmaProp value representing public key of Diffie Hellman signature protocol.
/// Common input: (g,h,u,v)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ProveDHTuple {
    gv: Box<EcPoint>,
    hv: Box<EcPoint>,
    uv: Box<EcPoint>,
    vv: Box<EcPoint>,
}

/// Sigma proposition
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SigmaProofOfKnowledgeTree {
    /// public key of Diffie Hellman signature protocol
    ProveDHTuple(ProveDHTuple),
    /// public key of discrete logarithm signature protocol
    ProveDlog(ProveDlog),
}

/// Algebraic data type of sigma proposition expressions
/// Values of this type are used as values of SigmaProp type
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SigmaBoolean {
    /// Represents boolean values (true/false)
    TrivialProp(bool),
    /// Sigma proposition
    ProofOfKnowledge(SigmaProofOfKnowledgeTree),
    /// AND conjunction for sigma propositions
    CAND(Vec<SigmaBoolean>),
}

impl SigmaBoolean {
    /// get OpCode for serialization
    pub fn op_code(&self) -> OpCode {
        match self {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(_)) => {
                OpCode::PROVE_DLOG
            }
            _ => todo!(),
        }
    }
}

/// Proposition which can be proven and verified by sigma protocol.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SigmaProp(SigmaBoolean);

impl SigmaProp {
    /// create new sigma propostion from [`SigmaBoolean`] value
    pub fn new(sbool: SigmaBoolean) -> Self {
        SigmaProp { 0: sbool }
    }

    /// get [`SigmaBoolean`] value
    pub fn value(&self) -> &SigmaBoolean {
        &self.0
    }
}

/// Proof tree
pub enum ProofTree {
    /// Unchecked tree
    UncheckedTree(UncheckedTree),
    /// Unproven tree
    UnprovenTree(UnprovenTree),
}

impl ProofTree {
    pub fn with_challenge(&self, challenge: Challenge) -> ProofTree {
        todo!()
    }
}

/// Unproven tree
pub enum UnprovenTree {
    UnprovenSchnorr(UnprovenSchnorr),
}

impl UnprovenTree {
    pub fn real(&self) -> bool {
        match self {
            UnprovenTree::UnprovenSchnorr(us) => !us.simulated,
        }
    }
}

pub struct UnprovenSchnorr {
    proposition: ProveDlog,
    commitment_opt: Option<FirstDlogProverMessage>,
    randomness_opt: Option<BigInteger>,
    challenge_opt: Option<Challenge>,
    simulated: bool,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Challenge(FiatShamirHash);

/// Unchecked sigma tree
pub enum UncheckedSigmaTree {
    UncheckedLeaf(UncheckedLeaf),
}

pub enum UncheckedLeaf {
    UncheckedSchnorr(UncheckedSchnorr),
}

impl UncheckedLeaf {
    pub fn proposition(&self) -> SigmaBoolean {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(us.proposition.clone()),
            ),
        }
    }
}

pub struct UncheckedSchnorr {
    proposition: ProveDlog,
    commitment_opt: Option<FirstDlogProverMessage>,
    challenge: Challenge,
    second_message: SecondDlogProverMessage,
}

impl UncheckedSigmaTree {
    // pub fn challenge(&self) -> Challenge {
    //     match self {
    //         UncheckedSigmaTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(us)) => us.challenge,
    //     }
    // }
}

/// Unchecked tree
pub enum UncheckedTree {
    /// No proof needed
    NoProof,
    /// Unchecked sigma tree
    UncheckedSigmaTree(UncheckedSigmaTree),
}

fn serialize_sig(tree: UncheckedTree) -> Vec<u8> {
    match tree {
        UncheckedTree::NoProof => vec![],
        UncheckedTree::UncheckedSigmaTree(_) => todo!(),
    }
}

///  Prover Step 7: Convert the tree to a string s for input to the Fiat-Shamir hash function.
///  The conversion should be such that the tree can be unambiguously parsed and restored given the string.
///  For each non-leaf node, the string should contain its type (OR or AND).
///  For each leaf node, the string should contain the Sigma-protocol statement being proven and the commitment.
///  The string should not contain information on whether a node is marked "real" or "simulated",
///  and should not contain challenges, responses, or the real/simulated flag for any node.
fn fiat_shamir_tree_to_bytes(tree: &ProofTree) -> Vec<u8> {
    // let propTree = ErgoTree.withSegregation(SigmaPropConstant(l.proposition))
    // val propBytes = DefaultSerializer.serializeErgoTree(propTree)
    // val commitmentBytes = l.commitmentOpt.get.bytes
    // leafPrefix +:
    //   ((Shorts.toByteArray(propBytes.length.toShort) ++ propBytes) ++
    //     (Shorts.toByteArray(commitmentBytes.length.toShort) ++ commitmentBytes))
    todo!()
}

/** Size of the binary representation of any group element (2 ^ groupSizeBits == <number of elements in a group>) */
pub const GROUP_SIZE_BITS: usize = 256;
/** Number of bytes to represent any group element as byte array */
pub const GROUP_SIZE: usize = GROUP_SIZE_BITS / 8;
/** A size of challenge in Sigma protocols, in bits.
 * If this anything but 192, threshold won't work, because we have polynomials over GF(2^192) and no others.
 * So DO NOT change the value without implementing polynomials over GF(2^soundnessBits) first
 * and changing code that calls on GF2_192 and GF2_192_Poly classes!!!
 * We get the challenge by reducing hash function output to proper value.
 */
pub const SOUNDNESS_BITS: usize = 192;
pub const SOUNDNESS_BYTES: usize = SOUNDNESS_BITS / 8;

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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for ProveDlog {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<EcPoint>()).prop_map(ProveDlog::new).boxed()
        }
    }

    impl Arbitrary for SigmaBoolean {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<ProveDlog>())
                .prop_map(|p| {
                    SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(p))
                })
                .boxed()
        }
    }

    impl Arbitrary for SigmaProp {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<SigmaBoolean>()).prop_map(SigmaProp).boxed()
        }
    }

    #[test]
    fn ensure_soundness_bits() {
        // see SOUNDNESS_BITS doc comment
        assert!(SOUNDNESS_BITS < GROUP_SIZE_BITS);
        // blake2b hash function requirements
        assert!(SOUNDNESS_BYTES * 8 <= 512);
        assert!(SOUNDNESS_BYTES % 8 == 0);
    }

    // #[test]
    // fn fiat_shamir_tree_to_bytes_smoke_test() {
    //     let tree = ProofTree
    // }
}
