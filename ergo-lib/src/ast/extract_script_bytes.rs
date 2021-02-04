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

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractScriptBytes {
    input: Box<Expr>,
}

impl ExtractScriptBytes {
    pub const OP_CODE: OpCode = OpCode::EXTRACT_SCRIPT_BYTES;

    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SBox)?;
        Ok(ExtractScriptBytes {
            input: input.into(),
        })
    }

    pub fn tpe(&self) -> SType {
        SType::SColl(SType::SByte.into())
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl Evaluable for ExtractScriptBytes {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => Ok(b.ergo_tree.sigma_serialize_bytes().into()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractScriptBytes input to be Value::CBox, got {0:?}",
                input_v
            ))),
        }
    }
}

impl SigmaSerializable for ExtractScriptBytes {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(ExtractScriptBytes::new(Expr::sigma_parse(r)?)?)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::global_vars::GlobalVars;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;

    use super::*;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = ExtractScriptBytes {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }

    #[test]
    fn eval() {
        let e: Expr = ExtractScriptBytes {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Vec<u8>>(&e, ctx.clone()),
            ctx.self_box.ergo_tree.sigma_serialize_bytes()
        );
    }
}
