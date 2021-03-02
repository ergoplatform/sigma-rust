use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::mir::value::Value;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OptionGet {
    input: Box<Expr>,
}

impl OptionGet {
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
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
#[cfg(feature = "arbitrary")]
mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::mir::expr::Expr;
    use crate::mir::extract_reg_as::ExtractRegisterAs;
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;
    use crate::types::stype::SType;

    use super::OptionGet;

    #[test]
    fn eval_get() {
        let get_reg_expr: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SLong.into()),
        )
        .unwrap()
        .into();
        let option_get_expr: Expr = OptionGet {
            input: Box::new(get_reg_expr),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        let v = eval_out::<i64>(&option_get_expr, ctx.clone());
        assert_eq!(v, ctx.self_box.get_box(&ctx.box_arena).unwrap().value());
    }

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
