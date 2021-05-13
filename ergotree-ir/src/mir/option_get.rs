use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::UnaryOp;
use super::unary_op::UnaryOpTryBuild;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

/// Returns the Option's value or error if no value
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OptionGet {
    /// Object of SOption type
    pub input: Box<Expr>,
}

impl OptionGet {
    /// Type
    pub fn tpe(&self) -> SType {
        match self.input.tpe() {
            SType::SOption(o) => *o,
            _ => panic!(
                "expected OptionGet::input type to be SOption, got: {0:?}",
                self.input.tpe()
            ),
        }
    }
}

impl HasStaticOpCode for OptionGet {
    const OP_CODE: OpCode = OpCode::OPTION_GET;
}

impl UnaryOp for OptionGet {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl UnaryOpTryBuild for OptionGet {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        match input.post_eval_tpe() {
            SType::SOption(_) => Ok(OptionGet {
                input: Box::new(input),
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
        let e: Expr = OptionGet {
            input: Box::new(get_reg_expr),
        }
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
