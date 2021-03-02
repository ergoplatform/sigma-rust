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
pub struct ExtractAmount {
    input: Box<Expr>,
}

impl ExtractAmount {
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SBox)?;
        Ok(ExtractAmount {
            input: input.into(),
        })
    }

    pub fn tpe(&self) -> SType {
        SType::SLong
    }

    pub fn op_code(&self) -> OpCode {
        OpCode::EXTRACT_AMOUNT
    }
}

impl Evaluable for ExtractAmount {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => Ok(Value::Long(ctx.ctx.box_arena.get(&b)?.value())),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractAmount input to be Value::CBox, got {0:?}",
                input_v
            ))),
        }
    }
}

impl SigmaSerializable for ExtractAmount {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(ExtractAmount {
            input: Expr::sigma_parse(r)?.into(),
        })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = ExtractAmount {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
