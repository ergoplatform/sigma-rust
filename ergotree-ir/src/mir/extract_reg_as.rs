use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// Get box register value (Box.R0 - R9)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractRegisterAs {
    /// Box
    pub input: Box<Expr>,
    /// Register id to extract value from (0 is R0 .. 9 for R9)
    pub register_id: i8,
    /// Result type, to be wrapped in SOption
    pub elem_tpe: SType,
}

impl ExtractRegisterAs {
    pub(crate) const OP_CODE: OpCode = OpCode::EXTRACT_REGISTER_AS;

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, register_id: i8, tpe: SType) -> Result<Self, InvalidArgumentError> {
        if input.post_eval_tpe() != SType::SBox {
            return Err(InvalidArgumentError(format!(
                "expected input to be SBox, got {0:?}",
                input
            )));
        }
        let elem_tpe = match tpe {
            SType::SOption(t) => Ok(*t),
            _ => Err(InvalidArgumentError(format!(
                "expected tpe to be SOption, got {0:?}",
                tpe
            ))),
        }?;

        Ok(ExtractRegisterAs {
            input: input.into(),
            register_id,
            elem_tpe,
        })
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SOption(self.elem_tpe.clone().into())
    }
}

impl SigmaSerializable for ExtractRegisterAs {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        w.put_i8(self.register_id)?;
        self.elem_tpe.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let register_id = r.get_i8()?;
        let elem_tpe = SType::sigma_parse(r)?;
        Ok(ExtractRegisterAs::new(
            input,
            register_id,
            SType::SOption(elem_tpe.into()),
        )?)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SLong.into()),
        )
        .unwrap()
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
