use crate::serialization::op_code::OpCode;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Context type instance
pub enum ContextM {
    DataInputs,
}

impl ContextM {
    pub fn op_code(&self) -> OpCode {
        match self {
            ContextM::DataInputs => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for ContextM {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            use ContextM::*;
            prop_oneof![Just(DataInputs)].boxed()
        }
    }
}
