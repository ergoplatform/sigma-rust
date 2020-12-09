use crate::serialization::op_code::OpCode;

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
    pub fn op_code(&self) -> OpCode {
        match self {
            GlobalVars::SelfBox => OpCode::SELF_BOX,
            GlobalVars::Inputs => OpCode::INPUTS,
            GlobalVars::Outputs => OpCode::OUTPUTS,
            GlobalVars::Height => OpCode::HEIGHT,
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
