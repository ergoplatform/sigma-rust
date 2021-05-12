use crate::serialization::op_code::OpCode;

pub(crate) trait HasOpCode {
    const OP_CODE: OpCode;

    fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}
