//! Sigma protocols

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(missing_docs)]

pub mod dlog_group;
pub mod dlog_protocol;
pub mod prover;
pub mod sig_serializer;
pub mod verifier;

use k256::Scalar;

use crate::{
    ast::Expr,
    serialization::{op_code::OpCode, SigmaSerializable},
    ErgoTree,
};
use blake2::digest::{Update, VariableOutput};
use blake2::VarBlake2b;
use dlog_group::EcPoint;
use dlog_protocol::{FirstDlogProverMessage, SecondDlogProverMessage};
use std::{
    convert::{TryFrom, TryInto},
    rc::Rc,
};
use thiserror::Error;

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

pub trait ProverMessage {
    fn bytes(&self) -> Vec<u8>;
}

pub enum FirstProverMessage {
    FirstDlogProverMessage(FirstDlogProverMessage),
    FirstDHTProverMessage,
}

impl ProverMessage for FirstProverMessage {
    fn bytes(&self) -> Vec<u8> {
        match self {
            FirstProverMessage::FirstDlogProverMessage(fdpm) => fdpm.bytes(),
            FirstProverMessage::FirstDHTProverMessage => todo!(),
        }
    }
}

impl From<FirstDlogProverMessage> for FirstProverMessage {
    fn from(v: FirstDlogProverMessage) -> Self {
        FirstProverMessage::FirstDlogProverMessage(v)
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
        match self {
            ProofTree::UncheckedTree(_) => todo!(),
            ProofTree::UnprovenTree(ut) => match ut {
                UnprovenTree::UnprovenLeaf(ul) => match ul {
                    UnprovenLeaf::UnprovenSchnorr(us) => {
                        ProofTree::UnprovenTree(UnprovenTree::UnprovenLeaf(
                            UnprovenLeaf::UnprovenSchnorr(UnprovenSchnorr {
                                challenge_opt: Some(challenge),
                                ..us.clone()
                            }),
                        ))
                    }
                },
            },
        }
    }
}

impl From<UncheckedSigmaTree> for ProofTree {
    fn from(ust: UncheckedSigmaTree) -> Self {
        ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(ust))
    }
}

/// Unproven tree
pub enum UnprovenTree {
    UnprovenLeaf(UnprovenLeaf),
    // UnprovenConjecture,
}

impl UnprovenTree {
    pub fn real(&self) -> bool {
        match self {
            UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenSchnorr(us)) => !us.simulated,
            // UnprovenTree::UnprovenConjecture => todo!(),
        }
    }
}

pub enum UnprovenLeaf {
    UnprovenSchnorr(UnprovenSchnorr),
}

pub trait ProofTreeLeaf {
    fn proposition(&self) -> SigmaBoolean;

    fn commitment_opt(&self) -> Option<FirstProverMessage>;
}

impl ProofTreeLeaf for UnprovenLeaf {
    fn proposition(&self) -> SigmaBoolean {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(us.proposition.clone()),
            ),
        }
    }

    fn commitment_opt(&self) -> Option<FirstProverMessage> {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => Some(FirstProverMessage::FirstDlogProverMessage(
                FirstDlogProverMessage(*us.proposition.h.clone()),
            )),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnprovenSchnorr {
    proposition: ProveDlog,
    commitment_opt: Option<FirstDlogProverMessage>,
    randomness_opt: Option<Scalar>,
    challenge_opt: Option<Challenge>,
    simulated: bool,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Challenge(FiatShamirHash);

impl Into<Scalar> for Challenge {
    fn into(self) -> Scalar {
        let v: [u8; SOUNDNESS_BYTES] = self.0.into();
        // prepend zeroes to 32 bytes (big-endian)
        let mut prefix = vec![0u8; 8];
        prefix.append(&mut v.to_vec());
        Scalar::from_bytes_reduced(prefix.as_slice().try_into().expect("32 bytes"))
    }
}

impl Into<Vec<u8>> for Challenge {
    fn into(self) -> Vec<u8> {
        let arr: [u8; SOUNDNESS_BYTES] = self.0.into();
        arr.to_vec()
    }
}

impl From<FiatShamirHash> for Challenge {
    fn from(fsh: FiatShamirHash) -> Self {
        Challenge(fsh)
    }
}

/// Unchecked sigma tree
#[derive(PartialEq, Debug, Clone)]
pub enum UncheckedSigmaTree {
    UncheckedLeaf(UncheckedLeaf),
    UncheckedConjecture,
}

impl UncheckedSigmaTree {
    pub fn challenge(&self) -> Challenge {
        match self {
            UncheckedSigmaTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(us)) => {
                us.challenge.clone()
            }
            UncheckedSigmaTree::UncheckedConjecture => todo!(),
        }
    }
}

impl From<UncheckedSchnorr> for UncheckedSigmaTree {
    fn from(us: UncheckedSchnorr) -> Self {
        UncheckedSigmaTree::UncheckedLeaf(us.into())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum UncheckedLeaf {
    UncheckedSchnorr(UncheckedSchnorr),
}

impl ProofTreeLeaf for UncheckedLeaf {
    fn proposition(&self) -> SigmaBoolean {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(us.proposition.clone()),
            ),
        }
    }
    fn commitment_opt(&self) -> Option<FirstProverMessage> {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => us.commitment_opt.clone().map(Into::into),
        }
    }
}

impl From<UncheckedSchnorr> for UncheckedLeaf {
    fn from(us: UncheckedSchnorr) -> Self {
        UncheckedLeaf::UncheckedSchnorr(us)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct UncheckedSchnorr {
    proposition: ProveDlog,
    commitment_opt: Option<FirstDlogProverMessage>,
    challenge: Challenge,
    second_message: SecondDlogProverMessage,
}

/// Unchecked tree
pub enum UncheckedTree {
    /// No proof needed
    NoProof,
    /// Unchecked sigma tree
    UncheckedSigmaTree(UncheckedSigmaTree),
}

impl From<UncheckedSchnorr> for UncheckedTree {
    fn from(us: UncheckedSchnorr) -> Self {
        UncheckedTree::UncheckedSigmaTree(us.into())
    }
}

///  Prover Step 7: Convert the tree to a string s for input to the Fiat-Shamir hash function.
///  The conversion should be such that the tree can be unambiguously parsed and restored given the string.
///  For each non-leaf node, the string should contain its type (OR or AND).
///  For each leaf node, the string should contain the Sigma-protocol statement being proven and the commitment.
///  The string should not contain information on whether a node is marked "real" or "simulated",
///  and should not contain challenges, responses, or the real/simulated flag for any node.
fn fiat_shamir_tree_to_bytes(tree: &ProofTree) -> Vec<u8> {
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

/** Size of the binary representation of any group element (2 ^ groupSizeBits == <number of elements in a group>) */
pub const GROUP_SIZE_BITS: usize = 256;
/** Number of bytes to represent any group element as byte array */
pub const GROUP_SIZE: usize = GROUP_SIZE_BITS / 8;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GroupSizeBytes(pub Box<[u8; GROUP_SIZE]>);

impl From<GroupSizeBytes> for Scalar {
    fn from(b: GroupSizeBytes) -> Self {
        let sl: &[u8] = b.0.as_ref();
        Scalar::from_bytes_reduced(sl.try_into().expect(""))
    }
}

impl From<&[u8; GROUP_SIZE]> for GroupSizeBytes {
    fn from(b: &[u8; GROUP_SIZE]) -> Self {
        GroupSizeBytes(Box::new(*b))
    }
}

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
}
