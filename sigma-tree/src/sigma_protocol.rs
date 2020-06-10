use k256::arithmetic::Scalar;

use crate::{ecpoint::EcPoint, serialization::op_code::OpCode};
use std::convert::TryInto;

pub struct DlogProverInput {
    pub w: Scalar,
}

impl DlogProverInput {
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

    #[allow(dead_code)]
    fn public_image(&self) -> ProveDlog {
        // TODO: test and remove annot(dead_code)
        let g = EcPoint::generator();
        ProveDlog::new(g.exponentiate(&self.w))
    }
}

pub trait PrivateInput {
    fn public_image(&self) -> SigmaProofOfKnowledgeTree;
}

impl PrivateInput for DlogProverInput {
    #[allow(dead_code)]
    fn public_image(&self) -> SigmaProofOfKnowledgeTree {
        // TODO: test and remove annot(dead_code)
        SigmaProofOfKnowledgeTree::ProveDlog(self.public_image())
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ProveDlog {
    pub h: Box<EcPoint>,
}

impl ProveDlog {
    pub fn new(ecpoint: EcPoint) -> ProveDlog {
        ProveDlog {
            h: Box::new(ecpoint),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ProveDHTuple {
    gv: Box<EcPoint>,
    hv: Box<EcPoint>,
    uv: Box<EcPoint>,
    vv: Box<EcPoint>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SigmaProofOfKnowledgeTree {
    ProveDHTuple(ProveDHTuple),
    ProveDlog(ProveDlog),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SigmaBoolean {
    ProofOfKnowledge(SigmaProofOfKnowledgeTree),
    CAND(Vec<SigmaBoolean>),
}

impl SigmaBoolean {
    pub fn op_code(&self) -> OpCode {
        match self {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(_)) => {
                OpCode::PROVE_DLOG
            }
            _ => todo!(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SigmaProp(SigmaBoolean);

impl SigmaProp {
    pub fn new(sbool: SigmaBoolean) -> Self {
        SigmaProp { 0: sbool }
    }

    pub fn value(&self) -> &SigmaBoolean {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for SigmaBoolean {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<EcPoint>())
                .prop_map(|ecp| {
                    SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(
                        ProveDlog::new(ecp),
                    ))
                })
                .boxed()
        }
    }

    impl Arbitrary for SigmaProp {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<SigmaBoolean>()).prop_map(|sb| SigmaProp(sb)).boxed()
        }
    }
}
