#![allow(missing_docs)]

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use sigma_ser::vlq_encode;

use std::io;
use vlq_encode::WriteSigmaVlqExt;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct OpCode(u8);

impl OpCode {
    // reference implementation
    // https://github.com/ScorexFoundation/sigmastate-interpreter/blob/develop/sigmastate/src/main/scala/sigmastate/serialization/OpCodes.scala

    pub const LAST_DATA_TYPE: OpCode = OpCode(111);
    pub const LAST_CONSTANT_CODE: OpCode = OpCode(Self::LAST_DATA_TYPE.value() + 1);

    pub const CONSTANT_PLACEHOLDER: OpCode = Self::new_op_code(3);

    /// Environment (context methods)
    pub const HEIGHT: OpCode = Self::new_op_code(51);
    pub const INPUTS: OpCode = Self::new_op_code(52);
    pub const OUTPUTS: OpCode = Self::new_op_code(53);
    pub const SELF_BOX: OpCode = Self::new_op_code(55);

    pub const FOLD: OpCode = Self::new_op_code(64);
    pub const PROVE_DLOG: OpCode = Self::new_op_code(93);

    pub const PROPERTY_CALL: OpCode = Self::new_op_code(107);
    pub const METHOD_CALL: OpCode = Self::new_op_code(108);

    pub const CONTEXT: OpCode = Self::new_op_code(142);

    const fn new_op_code(shift: u8) -> OpCode {
        OpCode(Self::LAST_CONSTANT_CODE.value() + shift)
    }

    pub fn parse(b: u8) -> OpCode {
        OpCode(b)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

impl SigmaSerializable for OpCode {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u8(self.0)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let code = r.get_u8()?;
        Ok(OpCode::parse(code))
    }
}
