use crate::chain::ergo_box::ErgoBox;
use crate::chain::ergo_box::RegisterId;
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

use super::constant::TryExtractInto;
use super::expr::Expr;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractRegisterAs {
    /// Box
    pub input: Expr,
    /// Register id to extract value from
    pub register_id: RegisterId,
    /// Type
    pub tpe: SType,
}

impl ExtractRegisterAs {
    pub fn op_code(&self) -> OpCode {
        OpCode::EXTRACT_REGISTER_AS
    }
}

impl Evaluable for ExtractRegisterAs {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Ok(Value::Opt(Box::new(
            self.input
                .eval(env, ctx)?
                .try_extract_into::<ErgoBox>()?
                .get_register(self.register_id)
                .map(|c| c.v),
        )))
    }
}

impl SigmaSerializable for ExtractRegisterAs {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.register_id.sigma_serialize(w)?;
        self.tpe.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let register_id = RegisterId::sigma_parse(r)?;
        let tpe = SType::sigma_parse(r)?;
        Ok(ExtractRegisterAs {
            input,
            register_id,
            tpe,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::global_vars::GlobalVars;
    use crate::ast::option_get::OptionGet;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_box_get_reg() {
        let get_reg_expr: Expr = Box::new(ExtractRegisterAs {
            input: Box::new(GlobalVars::SelfBox).into(),
            register_id: RegisterId::R0,
            tpe: SType::SOption(SType::SLong.into()),
        })
        .into();
        let option_get_expr: Expr = Box::new(OptionGet::new(get_reg_expr).unwrap()).into();
        let ctx = Rc::new(force_any_val::<Context>());
        let v = eval_out::<i64>(&option_get_expr, ctx.clone());
        assert_eq!(v, ctx.self_box.value.as_i64());
    }

    #[test]
    fn ser_roundtrip() {
        let e: Expr = Box::new(ExtractRegisterAs {
            input: Box::new(GlobalVars::SelfBox).into(),
            register_id: RegisterId::R0,
            tpe: SType::SOption(SType::SLong.into()),
        })
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
