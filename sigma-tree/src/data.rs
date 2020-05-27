//! Underlying Sigma data types

use crate::{ecpoint::EcPoint, serialization::op_code::OpCode};

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug)]
pub enum SigmaBoolean {
    ProveDHTuple {
        gv: EcPoint,
        hv: EcPoint,
        uv: EcPoint,
        vv: EcPoint,
    },
    ProveDlog(EcPoint),
    CAND(Vec<SigmaBoolean>),
}

impl SigmaBoolean {
    pub fn op_code(&self) -> OpCode {
        match self {
            SigmaBoolean::ProveDHTuple { .. } => todo!(),
            SigmaBoolean::ProveDlog(_) => OpCode::PROVE_DLOG,
            SigmaBoolean::CAND(_) => todo!(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
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
                .prop_map(|ecp| SigmaBoolean::ProveDlog(ecp))
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
