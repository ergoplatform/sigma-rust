use ergotree_ir::bigint256::BigInt256;
use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::upcast::Upcast;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

fn upcast_to_bigint<'a>(in_v: Value<'a>, ctx: &Context) -> Result<Value<'a>, EvalError> {
    match in_v {
        Value::Byte(v) => Ok(BigInt256::from(v).into()),
        Value::Short(v) => Ok(BigInt256::from(v).into()),
        Value::Int(v) => Ok(BigInt256::from(v).into()),
        Value::Long(v) => Ok(BigInt256::from(v).into()),
        Value::BigInt(_) if ctx.tree_version() >= ErgoTreeVersion::V3 => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to BigInt",
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

/// Numeric "width" used to coerce mismatched arithmetic operands to the wider
/// type, ordered Byte<Short<Int<Long<BigInt. Returns `None` for `UnsignedBigInt`
/// and non-numeric values, which keep the existing same-type-only arith path.
pub(crate) fn numeric_width(v: &Value) -> Option<u8> {
    match v {
        Value::Byte(_) => Some(0),
        Value::Short(_) => Some(1),
        Value::Int(_) => Some(2),
        Value::Long(_) => Some(3),
        Value::BigInt(_) => Some(4),
        _ => None,
    }
}

/// Coerce a numeric `Value` up to the given `width` (the wider operand's), reusing
/// the `Upcast` conversions. Pure conversion. Only ever called on the narrower
/// operand, so the BigInt arm never gets a BigInt input.
pub(crate) fn coerce_numeric_to<'a>(
    v: Value<'a>,
    width: u8,
    ctx: &Context,
) -> Result<Value<'a>, EvalError> {
    match width {
        4 => upcast_to_bigint(v, ctx),
        3 => upcast_to_long(v),
        2 => upcast_to_int(v),
        1 => upcast_to_short(v),
        _ => Ok(v),
    }
}

impl Evaluable for Upcast {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match self.tpe {
            SType::SBigInt => upcast_to_bigint(input_v, ctx),
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

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergotree_ir::mir::constant::Constant;
    use sigma_test_util::force_any_val;

    use crate::eval::test_util::{eval_out_wo_ctx, try_eval_out_with_version};

    use super::*;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn from_byte(v in any::<i8>()) {
            let c: Constant = v.into();
            assert_eq!(
                eval_out_wo_ctx::<i8>(&Upcast::new(c.clone().into(), SType::SByte).unwrap().into()),
                v
            );
            assert_eq!(
                eval_out_wo_ctx::<i16>(&Upcast::new(c.clone().into(), SType::SShort).unwrap().into()),
                v as i16
            );
            assert_eq!(
                eval_out_wo_ctx::<i32>(&Upcast::new(c.clone().into(), SType::SInt).unwrap().into()),
                v as i32
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&Upcast::new(c.clone().into(), SType::SLong).unwrap().into()),
                v as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(&Upcast::new(c.into(), SType::SBigInt).unwrap().into()),
                v.into()
            );
        }

        #[test]
        fn from_short(v in any::<i16>()) {
            let c: Constant = v.into();
            assert_eq!(
                eval_out_wo_ctx::<i16>(&Upcast::new(c.clone().into(), SType::SShort).unwrap().into()),
                v
            );
            assert_eq!(
                eval_out_wo_ctx::<i32>(&Upcast::new(c.clone().into(), SType::SInt).unwrap().into()),
                v as i32
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&Upcast::new(c.clone().into(), SType::SLong).unwrap().into()),
                v as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(&Upcast::new(c.into(), SType::SBigInt).unwrap().into()),
                v.into()
            );
        }

        #[test]
        fn from_int(v in any::<i32>()) {
            let c: Constant = v.into();
            assert_eq!(
                eval_out_wo_ctx::<i32>(&Upcast::new(c.clone().into(), SType::SInt).unwrap().into()),
                v
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&Upcast::new(c.clone().into(), SType::SLong).unwrap().into()),
                v as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(&Upcast::new(c.into(), SType::SBigInt).unwrap().into()),
                v.into()
            );
        }

        #[test]
        fn from_long(v in any::<i64>()) {
            let c: Constant = v.into();
            assert_eq!(
                eval_out_wo_ctx::<i64>(&Upcast::new(c.clone().into(), SType::SLong).unwrap().into()),
                v
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(&Upcast::new(c.into(), SType::SBigInt).unwrap().into()),
                v.into()
            );
        }

        #[test]
        fn from_bigint(v in any::<BigInt256>()) {
            let c: Constant = v.into();
            let ctx = force_any_val::<Context>();
            (0..ErgoTreeVersion::V3.into()).for_each(|version| {
                assert!(try_eval_out_with_version::<BigInt256>(&Upcast::new(c.clone().into(), SType::SBigInt).unwrap().into(), &ctx, version, version).is_err());
            });
            (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(|version| {
                assert_eq!(try_eval_out_with_version::<BigInt256>(&Upcast::new(c.clone().into(), SType::SBigInt).unwrap().into(), &ctx, version, version).unwrap(), v.clone());
            });
        }
    }
}
