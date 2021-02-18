//! Global variables

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
}

impl GlobalVars {
    /// Op code (serialization)
    pub fn op_code(&self) -> OpCode {
        match self {
            GlobalVars::SelfBox => OpCode::SELF_BOX,
            GlobalVars::Inputs => OpCode::INPUTS,
            GlobalVars::Outputs => OpCode::OUTPUTS,
            GlobalVars::Height => OpCode::HEIGHT,
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        match self {
            GlobalVars::Inputs => SType::SColl(Box::new(SType::SBox)),
            GlobalVars::Outputs => SType::SColl(Box::new(SType::SBox)),
            GlobalVars::Height => SType::SInt,
            GlobalVars::SelfBox => SType::SBox,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for GlobalVars {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            use GlobalVars::*;
            prop_oneof![Just(Inputs), Just(Outputs), Just(Height), Just(SelfBox),].boxed()
        }
    }
}
