use crate::serialization::op_code::OpCode;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Context type instance
pub enum ContextM {
    /// Tx inputs
    Inputs,
    /// Tx outputs
    Outputs,
    /// Current blockchain height
    Height,
    /// ErgoBox instance, which script is being evaluated
    SelfBox,
    /// Tx data inputs
    DataInputs,
}

impl ContextM {
    pub fn op_code(&self) -> OpCode {
        match self {
            ContextM::SelfBox => OpCode::SELF_BOX,
            _ => todo!(),
        }
    }
}
