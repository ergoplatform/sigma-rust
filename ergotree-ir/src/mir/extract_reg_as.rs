use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::ir_ergo_box::IrBoxId;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::constant::TryExtractInto;
use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractRegisterAs {
    /// Box
    input: Box<Expr>,
    /// Register id to extract value from
    register_id: i8,
    /// Result type, to be wrapped in SOption
    elem_tpe: SType,
}

impl ExtractRegisterAs {
    pub const OP_CODE: OpCode = OpCode::EXTRACT_REGISTER_AS;

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

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }

    pub fn tpe(&self) -> SType {
        SType::SOption(self.elem_tpe.clone().into())
    }
}

impl Evaluable for ExtractRegisterAs {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let ir_box_id = self.input.eval(env, ctx)?.try_extract_into::<IrBoxId>()?;
        Ok(Value::Opt(Box::new(
            ctx.ctx
                .box_arena
                .get(&ir_box_id)?
                .get_register(self.register_id)
                .map(|c| c.v),
        )))
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
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::mir::global_vars::GlobalVars;
    use crate::mir::option_get::OptionGet;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_box_get_reg() {
        let get_reg_expr: Expr = ExtractRegisterAs::new(
            GlobalVars::SelfBox.into(),
            0,
            SType::SOption(SType::SLong.into()),
        )
        .unwrap()
        .into();
        let option_get_expr: Expr = OptionGet::new(get_reg_expr).unwrap().into();
        let ctx = Rc::new(force_any_val::<Context>());
        let v = eval_out::<i64>(&option_get_expr, ctx.clone());
        assert_eq!(v, ctx.self_box.get_box(&ctx.box_arena).unwrap().value());
    }

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
