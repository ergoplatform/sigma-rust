use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

/// Placeholder for a constant in ErgoTree.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConstantPlaceholder {
    /// Zero based index in ErgoTree.constants array.
    pub id: u32,
    /// Type of the constant value
    pub tpe: SType,
}

impl ConstantPlaceholder {}

impl HasStaticOpCode for ConstantPlaceholder {
    const OP_CODE: OpCode = OpCode::CONSTANT_PLACEHOLDER;
}
