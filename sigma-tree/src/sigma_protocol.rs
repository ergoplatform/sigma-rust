//! Sigma protocols

pub mod prover;
pub mod verifier;

use k256::arithmetic::Scalar;

use crate::{ecpoint::EcPoint, serialization::op_code::OpCode};
use std::convert::TryInto;

/// Secret key of discrete logarithm signature protocol
pub struct DlogProverInput {
    /// secret key value
    pub w: Scalar,
}

impl DlogProverInput {
    /// generates random secret in the range [0, n), where n is DLog group order.
    pub fn random() -> DlogProverInput {
        let scalar = loop {
            // Generate a new secret key using the operating system's
            // cryptographically secure random number generator
            let sk = k256::SecretKey::generate();
            let bytes: [u8; 32] = sk
                .secret_scalar()
                .as_ref()
                .as_slice()
                .try_into()
                .expect("expected 32 bytes");
            // Returns None if the byte array does not contain
            // a big-endian integer in the range [0, n), where n is group order.
            let maybe_scalar = Scalar::from_bytes(bytes);
            if bool::from(maybe_scalar.is_some()) {
                break maybe_scalar.unwrap();
            }
        };
        DlogProverInput { w: scalar }
    }

    /// public key of discrete logarithm signature protocol
    fn public_image(&self) -> ProveDlog {
        // test it, see https://github.com/ergoplatform/sigma-rust/issues/38
        let g = EcPoint::generator();
        ProveDlog::new(g.exponentiate(&self.w))
    }
}

/// Get public key for signature protocol
pub trait PrivateInput {
    /// public key
    fn public_image(&self) -> SigmaProofOfKnowledgeTree;
}

impl PrivateInput for DlogProverInput {
    fn public_image(&self) -> SigmaProofOfKnowledgeTree {
        SigmaProofOfKnowledgeTree::ProveDlog(self.public_image())
    }
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

pub enum ProofTree {
    UncheckedTree(UncheckedTree),
    UnprovenTree(UnprovenTree),
}

pub enum UnprovenTree {}

pub enum UncheckedSigmaTree {}

pub enum UncheckedTree {
    NoProof,
    UncheckedSigmaTree(UncheckedSigmaTree),
}

fn serialize_sig(tree: UncheckedTree) -> Vec<u8> {
    todo!()
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
}
