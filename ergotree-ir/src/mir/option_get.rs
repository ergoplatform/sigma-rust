use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

/// Returns the Option's value or error if no value
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OptionGet {
    /// Object of SOption type
    pub input: Box<Expr>,
    /// Option element type
    elem_tpe: SType,
}

impl OptionGet {
    /// Type
    pub fn tpe(&self) -> SType {
        self.elem_tpe.clone()
    }
}

impl HasStaticOpCode for OptionGet {
    const OP_CODE: OpCode = OpCode::OPTION_GET;
}

impl OneArgOp for OptionGet {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl OneArgOpTryBuild for OptionGet {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        match input.post_eval_tpe() {
            SType::SOption(elem_tpe) => Ok(OptionGet {
                input: Box::new(input),
                elem_tpe: *elem_tpe,
            }),
            _ => Err(InvalidArgumentError(format!(
                "expected OptionGet::input type to be SOption, got: {0:?}",
                input.tpe(),
            ))),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
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
        let e: Expr = OptionGet::try_build(get_reg_expr).unwrap().into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
