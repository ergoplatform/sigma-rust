//! Sigma boolean types

use self::cand::Cand;
use self::cor::Cor;

use super::dlog_group::EcPoint;
use crate::ergo_tree::ErgoTree;
use crate::has_opcode::HasOpCode;
use crate::mir::constant::Constant;
use crate::mir::expr::Expr;
use crate::serialization::op_code::OpCode;
use crate::serialization::SigmaSerializable;
use std::convert::TryFrom;
use std::convert::TryInto;

extern crate derive_more;
use derive_more::From;
use derive_more::Into;
use derive_more::TryInto;

pub mod cand;
pub mod cor;

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

impl From<EcPoint> for ProveDlog {
    fn from(p: EcPoint) -> Self {
        ProveDlog::new(p)
    }
}

/// Construct a new SigmaProp value representing public key of Diffie Hellman signature protocol.
/// Common input: (g,h,u,v)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ProveDhTuple {
    /// Generator `g`
    pub gv: Box<EcPoint>,
    /// Point `g^x`
    pub hv: Box<EcPoint>,
    /// Point `g^y`
    pub uv: Box<EcPoint>,
    /// Point `g^xy`
    pub vv: Box<EcPoint>,
}

/// Sigma proposition
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub enum SigmaProofOfKnowledgeTree {
    /// public key of Diffie Hellman signature protocol
    ProveDhTuple(ProveDhTuple),
    /// public key of discrete logarithm signature protocol
    ProveDlog(ProveDlog),
}

/// Conjunctions for sigma propositions
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SigmaConjecture {
    /// AND
    Cand(Cand),
    /// OR
    Cor(Cor),
}

/// Algebraic data type of sigma proposition expressions
/// Values of this type are used as values of SigmaProp type
#[derive(PartialEq, Eq, Debug, Clone, From, TryInto)]
pub enum SigmaBoolean {
    /// Represents boolean values (true/false)
    TrivialProp(bool),
    /// Sigma proposition
    ProofOfKnowledge(SigmaProofOfKnowledgeTree),
    /// Conjunctions for sigma propositions
    SigmaConjecture(SigmaConjecture),
}

impl HasOpCode for SigmaBoolean {
    /// get OpCode for serialization
    fn op_code(&self) -> OpCode {
        match self {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(_)) => {
                OpCode::PROVE_DLOG
            }
            _ => todo!(),
        }
    }
}

/// Failed to extract specified underlying type from SigmaBoolean
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConversionError;

impl TryInto<ProveDlog> for SigmaBoolean {
    type Error = ConversionError;
    fn try_into(self) -> Result<ProveDlog, Self::Error> {
        match self {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pd)) => Ok(pd),
            _ => Err(ConversionError),
        }
    }
}

impl From<ProveDlog> for SigmaBoolean {
    fn from(v: ProveDlog) -> Self {
        SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(v))
    }
}

/// Proposition which can be proven and verified by sigma protocol.
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct SigmaProp(SigmaBoolean);

impl SigmaProp {
    /// create new sigma proposition from [`SigmaBoolean`] value
    pub fn new(sbool: SigmaBoolean) -> Self {
        SigmaProp { 0: sbool }
    }

    /// get [`SigmaBoolean`] value
    pub fn value(&self) -> &SigmaBoolean {
        &self.0
    }

    /// Serialized bytes of a SigmaProp value
    pub fn prop_bytes(&self) -> Vec<u8> {
        // in order to have comparisons like  `box.propositionBytes == pk.propBytes` we need to make sure
        // the same serialization method is used in both cases
        let c: Constant = self.clone().into();
        let e: Expr = c.into();
        let ergo_tree: ErgoTree = e.into();
        ergo_tree.sigma_serialize_bytes()
    }
}

impl TryFrom<SigmaProp> for bool {
    type Error = ConversionError;

    fn try_from(value: SigmaProp) -> Result<Self, Self::Error> {
        value.0.try_into().map_err(|_| ConversionError)
    }
}

impl From<ProveDlog> for SigmaProp {
    fn from(pd: ProveDlog) -> Self {
        SigmaProp(SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pd),
        ))
    }
}

impl From<ProveDhTuple> for SigmaProp {
    fn from(dh: ProveDhTuple) -> Self {
        SigmaProp(SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDhTuple(dh),
        ))
    }
}
/// Arbitrary impl for ProveDlog
#[cfg(feature = "arbitrary")]
mod arbitrary {
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
            (any::<SigmaBoolean>()).prop_map(SigmaProp::new).boxed()
        }
    }
}

#[cfg(test)]
mod tests {}
