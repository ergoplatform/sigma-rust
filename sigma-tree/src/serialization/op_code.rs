use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use sigma_ser::vlq_encode;

use std::io;
use vlq_encode::WriteSigmaVlqExt;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct OpCode(u8);

impl OpCode {
    pub const LAST_DATA_TYPE: OpCode = OpCode(111);
    pub const LAST_CONSTANT_CODE: OpCode = OpCode(Self::LAST_DATA_TYPE.value() + 1);

    pub const FOLD: OpCode = Self::new_op_code(64);
    pub const PROVE_DLOG: OpCode = Self::new_op_code(93);

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
