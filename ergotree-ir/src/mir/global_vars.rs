//! Global variables

use std::fmt::Display;

use crate::has_opcode::HasOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Predefined global variables
pub enum GlobalVars {
    /// Tx inputs
    Inputs,
    /// Tx outputs
    Outputs,
    /// Current blockchain height
    Height,
    /// ErgoBox instance, which script is being evaluated
    SelfBox,
    /// When interpreted evaluates to a ByteArrayConstant built from Context.minerPubkey
    MinerPubKey,
    /// GroupElement (EcPoint) generator
    GroupGenerator,
}

impl GlobalVars {
    /// Type
    pub fn tpe(&self) -> SType {
        match self {
            GlobalVars::Inputs => SType::SColl(Box::new(SType::SBox)),
            GlobalVars::Outputs => SType::SColl(Box::new(SType::SBox)),
            GlobalVars::Height => SType::SInt,
            GlobalVars::SelfBox => SType::SBox,
            GlobalVars::MinerPubKey => SType::SColl(Box::new(SType::SByte)),
            GlobalVars::GroupGenerator => SType::SGroupElement,
        }
    }
}

impl HasOpCode for GlobalVars {
    /// Op code (serialization)
    fn op_code(&self) -> OpCode {
        match self {
            GlobalVars::SelfBox => OpCode::SELF_BOX,
            GlobalVars::Inputs => OpCode::INPUTS,
            GlobalVars::Outputs => OpCode::OUTPUTS,
            GlobalVars::Height => OpCode::HEIGHT,
            GlobalVars::MinerPubKey => OpCode::MINER_PUBKEY,
            GlobalVars::GroupGenerator => OpCode::GROUP_GENERATOR,
        }
    }
}

impl Display for GlobalVars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalVars::SelfBox => write!(f, "SELF"),
            GlobalVars::Inputs => write!(f, "INPUTS"),
            GlobalVars::Outputs => write!(f, "OUTPUTS"),
            GlobalVars::Height => write!(f, "HEIGHT"),
            GlobalVars::MinerPubKey => write!(f, "MINER_PUBKEY"),
            GlobalVars::GroupGenerator => write!(f, "GROUP_GENERATOR"),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for GlobalVars {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            use GlobalVars::*;
            prop_oneof![
                Just(Inputs),
                Just(Outputs),
                Just(Height),
                Just(SelfBox),
                Just(MinerPubKey),
                Just(GroupGenerator)
            ]
            .boxed()
        }
    }
}
