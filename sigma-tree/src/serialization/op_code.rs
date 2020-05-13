use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;
use vlq_encode::WriteSigmaVlqExt;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct OpCode(u8);

impl OpCode {
    // TODO: use correct code
    pub const FOLD: OpCode = OpCode(0);

    pub fn parse(b: u8) -> OpCode {
        OpCode(b)
    }

    const fn assigned(b: u8) -> OpCode {
        OpCode(b)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

// TODO: correct value
pub const LAST_CONSTANT_CODE: OpCode = OpCode::assigned(0);

impl SigmaSerializable for OpCode {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        w.put_u8(self.0)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let code = r.get_u8()?;
        Ok(OpCode::parse(code))
    }
}

