use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::ast::value::Value;
use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OptionGet {
    input: Expr,
}

impl OptionGet {
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        match input.tpe() {
            SType::SOption(_) => Ok(OptionGet { input }),
            _ => Err(InvalidArgumentError(format!(
                "expected OptionGet::input type to be SOption, got: {0:?}",
                input.tpe(),
            ))),
        }
    }

    pub fn op_code(&self) -> OpCode {
        OpCode::OPTION_GET
    }

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

impl Evaluable for OptionGet {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let v = self.input.eval(env, ctx)?;
        match v {
            Value::Opt(opt_v) => {
                opt_v.ok_or_else(|| EvalError::NotFound("calling Option.get on None".to_string()))
            }
            _ => Err(EvalError::UnexpectedExpr(format!(
                "Don't know how to eval OptM: {0:?}",
                self
            ))),
        }
    }
}

impl SigmaSerializable for OptionGet {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(OptionGet::new(Expr::sigma_parse(r)?)?)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::expr::Expr;
    use crate::ast::extract_reg_as::ExtractRegisterAs;
    use crate::ast::global_vars::GlobalVars;
    use crate::chain::ergo_box::RegisterId;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;
    use crate::types::stype::SType;

    use super::OptionGet;

    #[test]
    fn eval_get() {
        let get_reg_expr: Expr = Box::new(ExtractRegisterAs {
            input: Box::new(GlobalVars::SelfBox).into(),
            register_id: RegisterId::R0,
            tpe: SType::SOption(SType::SLong.into()),
        })
        .into();
        let option_get_expr: Expr = Box::new(OptionGet {
            input: get_reg_expr,
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        let v = eval_out::<i64>(&option_get_expr, ctx.clone());
        assert_eq!(v, ctx.self_box.value.as_i64());
    }

    #[test]
    fn ser_roundtrip() {
        let get_reg_expr: Expr = Box::new(ExtractRegisterAs {
            input: Box::new(GlobalVars::SelfBox).into(),
            register_id: RegisterId::R0,
            tpe: SType::SOption(SType::SLong.into()),
        })
        .into();
        let e: Expr = Box::new(OptionGet {
            input: get_reg_expr,
        })
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
