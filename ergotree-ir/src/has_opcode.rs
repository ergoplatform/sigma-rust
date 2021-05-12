use crate::serialization::op_code::OpCode;

pub(crate) trait HasStaticOpCode {
    const OP_CODE: OpCode;
}

pub(crate) trait HasOpCode {
    fn op_code(&self) -> OpCode;
}

impl<T: HasStaticOpCode> HasOpCode for T {
    fn op_code(&self) -> OpCode {
        T::OP_CODE
    }
}
