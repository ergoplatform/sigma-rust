//! Sigma boolean types

use self::cand::Cand;
use self::cor::Cor;
use self::cthreshold::Cthreshold;

use super::dlog_group::EcPoint;
use crate::ergo_tree::{ErgoTree, ErgoTreeError};
use crate::has_opcode::{HasOpCode, HasStaticOpCode};
use crate::mir::constant::Constant;
use crate::mir::expr::Expr;
use crate::serialization::op_code::OpCode;
use crate::serialization::SigmaSerializable;
use std::convert::TryFrom;
use std::convert::TryInto;

extern crate derive_more;
use bounded_vec::BoundedVec;
use derive_more::From;
use derive_more::Into;
use derive_more::TryInto;

pub mod cand;
pub mod cor;
pub mod cthreshold;

/// Sigma conjecture items type with bounds check (2..=1000)
pub type SigmaConjectureItems<T> = BoundedVec<T, 2, 1000>;

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

impl HasStaticOpCode for ProveDlog {
    const OP_CODE: OpCode = OpCode::PROVE_DLOG;
}

impl From<EcPoint> for ProveDlog {
    fn from(p: EcPoint) -> Self {
        ProveDlog::new(p)
    }
}

/// Construct a new SigmaProp value representing public key of Diffie Hellman signature protocol.
/// Used in a proof that of equality of discrete logarithms (i.e., a proof of a Diffie-Hellman tuple):
/// given group elements g, h, u, v, the proof convinces a verifier that the prover knows `w` such
/// that `u = g^w` and `v = h^w`, without revealing `w`
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ProveDhTuple {
    /// Generator g
    pub g: Box<EcPoint>,
    /// Point h
    pub h: Box<EcPoint>,
    /// Point `u = g^w`
    pub u: Box<EcPoint>,
    /// Point `v= h^w`
    pub v: Box<EcPoint>,
}

impl HasStaticOpCode for ProveDhTuple {
    const OP_CODE: OpCode = OpCode::PROVE_DIFFIE_HELLMAN_TUPLE;
}

impl ProveDhTuple {
    /// Create new instance
    pub fn new(g: EcPoint, h: EcPoint, u: EcPoint, v: EcPoint) -> Self {
        Self {
            g: g.into(),
            h: h.into(),
            u: u.into(),
            v: v.into(),
        }
    }
}

/// Sigma proposition
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub enum SigmaProofOfKnowledgeTree {
    /// public key of Diffie Hellman signature protocol
    ProveDhTuple(ProveDhTuple),
    /// public key of discrete logarithm signature protocol
    ProveDlog(ProveDlog),
}

impl HasOpCode for SigmaProofOfKnowledgeTree {
    fn op_code(&self) -> OpCode {
        match self {
            SigmaProofOfKnowledgeTree::ProveDhTuple(dh) => dh.op_code(),
            SigmaProofOfKnowledgeTree::ProveDlog(dlog) => dlog.op_code(),
        }
    }
}

/// Conjunctions for sigma propositions
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SigmaConjecture {
    /// AND
    Cand(Cand),
    /// OR
    Cor(Cor),
    /// THRESHOLD
    Cthreshold(Cthreshold),
}

impl HasOpCode for SigmaConjecture {
    fn op_code(&self) -> OpCode {
        match self {
            SigmaConjecture::Cand(cand) => cand.op_code(),
            SigmaConjecture::Cor(cor) => cor.op_code(),
            SigmaConjecture::Cthreshold(ct) => ct.op_code(),
        }
    }
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
            SigmaBoolean::ProofOfKnowledge(kt) => kt.op_code(),
            SigmaBoolean::SigmaConjecture(sc) => sc.op_code(),
            SigmaBoolean::TrivialProp(tp) => {
                if *tp {
                    OpCode::TRIVIAL_PROP_TRUE
                } else {
                    OpCode::TRIVIAL_PROP_FALSE
                }
            }
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

impl TryInto<ProveDhTuple> for SigmaBoolean {
    type Error = ConversionError;
    fn try_into(self) -> Result<ProveDhTuple, Self::Error> {
        match self {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDhTuple(pdh)) => Ok(pdh),
            _ => Err(ConversionError),
        }
    }
}

impl From<ProveDhTuple> for SigmaBoolean {
    fn from(v: ProveDhTuple) -> Self {
        SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDhTuple(v))
    }
}

impl TryInto<Cand> for SigmaBoolean {
    type Error = ConversionError;
    fn try_into(self) -> Result<Cand, Self::Error> {
        match self {
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(c)) => Ok(c),
            _ => Err(ConversionError),
        }
    }
}

impl From<Cand> for SigmaBoolean {
    fn from(v: Cand) -> Self {
        SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(v))
    }
}

impl TryInto<Cor> for SigmaBoolean {
    type Error = ConversionError;
    fn try_into(self) -> Result<Cor, Self::Error> {
        match self {
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(c)) => Ok(c),
            _ => Err(ConversionError),
        }
    }
}

impl From<Cor> for SigmaBoolean {
    fn from(v: Cor) -> Self {
        SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(v))
    }
}

impl TryInto<Cthreshold> for SigmaBoolean {
    type Error = ConversionError;
    fn try_into(self) -> Result<Cthreshold, Self::Error> {
        match self {
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(c)) => Ok(c),
            _ => Err(ConversionError),
        }
    }
}

impl From<Cthreshold> for SigmaBoolean {
    fn from(v: Cthreshold) -> Self {
        SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(v))
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
    pub fn prop_bytes(&self) -> Result<Vec<u8>, ErgoTreeError> {
        // in order to have comparisons like  `box.propositionBytes == pk.propBytes` we need to make sure
        // the same serialization method is used in both cases
        let c: Constant = self.clone().into();
        let e: Expr = c.into();
        let ergo_tree: ErgoTree = e.try_into()?;
        Ok(ergo_tree.sigma_serialize_bytes()?)
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
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for ProveDlog {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<EcPoint>()).prop_map(ProveDlog::new).boxed()
        }
    }

    impl Arbitrary for ProveDhTuple {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<EcPoint>(),
                any::<EcPoint>(),
                any::<EcPoint>(),
                any::<EcPoint>(),
            )
                .prop_map(|(g, h, u, v)| ProveDhTuple::new(g, h, u, v))
                .boxed()
        }
    }

    pub fn primitive_type_value() -> BoxedStrategy<SigmaBoolean> {
        prop_oneof![
            any::<ProveDlog>().prop_map_into(),
            any::<ProveDhTuple>().prop_map_into(),
        ]
        .boxed()
    }

    impl Arbitrary for SigmaBoolean {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            primitive_type_value()
                .prop_recursive(1, 8, 4, |elem| {
                    prop_oneof![
                        vec(elem.clone(), 2..=4)
                            .prop_map(|elems| Cand {
                                items: elems.try_into().unwrap()
                            })
                            .prop_map_into(),
                        vec(elem, 2..=4)
                            .prop_map(|elems| Cor {
                                items: elems.try_into().unwrap()
                            })
                            .prop_map_into(),
                    ]
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

#[allow(clippy::panic)]
#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn sigma_boolean_ser_roundtrip(
            v in any::<SigmaBoolean>()) {
                prop_assert_eq![sigma_serialize_roundtrip(&v), v]
        }
    }
}
