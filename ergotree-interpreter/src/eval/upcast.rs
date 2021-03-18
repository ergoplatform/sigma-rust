use ergotree_ir::mir::upcast::Upcast;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;
use num_bigint::ToBigInt;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

fn upcast_to_bigint(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok(v.to_bigint().into()),
        Value::Short(v) => Ok(v.to_bigint().into()),
        Value::Int(v) => Ok(v.to_bigint().into()),
        Value::Long(v) => Ok(v.to_bigint().into()),
        Value::BigInt(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Long",
            in_v
        ))),
    }
}

fn upcast_to_long(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok((v as i64).into()),
        Value::Short(v) => Ok((v as i64).into()),
        Value::Int(v) => Ok((v as i64).into()),
        Value::Long(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Long",
            in_v
        ))),
    }
}

fn upcast_to_int(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok((v as i32).into()),
        Value::Short(v) => Ok((v as i32).into()),
        Value::Int(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Int",
            in_v
        ))),
    }
}

fn upcast_to_short(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok((v as i16).into()),
        Value::Short(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Short",
            in_v
        ))),
    }
}

fn upcast_to_byte(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Byte",
            in_v
        ))),
    }
}

impl Evaluable for Upcast {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match self.tpe {
            SType::SBigInt => upcast_to_bigint(input_v),
            SType::SLong => upcast_to_long(input_v),
            SType::SInt => upcast_to_int(input_v),
            SType::SShort => upcast_to_short(input_v),
            SType::SByte => upcast_to_byte(input_v),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Upcast: expected numeric value, got {0:?}",
                input_v
            ))),
        }
    }
}
