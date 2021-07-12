use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Returns the Option's value or error if no value
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OptionGetOrElse {
    /// Object of SOption type
    pub input: Box<Expr>,
    /// Default value if option is empty
    pub default: Box<Expr>,
    /// Option element type
    elem_tpe: SType,
}

impl OptionGetOrElse {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, default: Expr) -> Result<Self, InvalidArgumentError> {
        match input.post_eval_tpe() {
            SType::SOption(elem_tpe) => {
                default.check_post_eval_tpe(elem_tpe.as_ref())?;
                Ok(OptionGetOrElse {
                    input: Box::new(input),
                    default: Box::new(default),
                    elem_tpe: *elem_tpe,
                })
            }
            _ => Err(InvalidArgumentError(format!(
                "expected OptionGetOrElse::input type to be SOption, got: {0:?}",
                input.tpe(),
            ))),
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        self.elem_tpe.clone()
    }
}

impl HasStaticOpCode for OptionGetOrElse {
    const OP_CODE: OpCode = OpCode::OPTION_GET_OR_ELSE;
}

impl SigmaSerializable for OptionGetOrElse {
    fn sigma_serialize<W: SigmaByteWrite>(
        &self,
        w: &mut W,
    ) -> crate::serialization::SigmaSerializeResult {
        self.input.sigma_serialize(w)?;
        self.default.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let input = Expr::sigma_parse(r)?;
        let default = Expr::sigma_parse(r)?;
        Ok(OptionGetOrElse::new(input, default)?)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::mir::constant::Constant;
    use crate::mir::expr::Expr;
    use crate::mir::extract_reg_as::ExtractRegisterAs;
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::stype::SType;

    #[test]
    fn ser_roundtrip() {
        let get_reg_expr: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SLong.into()),
        )
        .unwrap()
        .into();
        let default_expr: Constant = 1i64.into();
        let e: Expr = OptionGetOrElse::new(get_reg_expr, default_expr.into())
            .unwrap()
            .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
