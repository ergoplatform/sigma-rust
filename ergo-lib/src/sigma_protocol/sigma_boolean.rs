//! Sigma boolean types

use super::dlog_group::EcPoint;
use crate::serialization::op_code::OpCode;
use std::convert::TryInto;

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

impl From<ProveDlog> for SigmaProofOfKnowledgeTree {
    fn from(pd: ProveDlog) -> Self {
        SigmaProofOfKnowledgeTree::ProveDlog(pd)
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

impl<T: Into<SigmaProofOfKnowledgeTree>> From<T> for SigmaBoolean {
    fn from(t: T) -> Self {
        SigmaBoolean::ProofOfKnowledge(t.into())
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

/// Proposition which can be proven and verified by sigma protocol.
#[derive(PartialEq, Eq, Debug, Clone)]
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
}

impl<T: Into<SigmaBoolean>> From<T> for SigmaProp {
    fn from(t: T) -> Self {
        SigmaProp(t.into())
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
            (any::<SigmaBoolean>()).prop_map(SigmaProp::new).boxed()
        }
    }
}
