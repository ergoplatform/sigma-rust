use ergotree_ir::bigint256::BigInt256;
use ergotree_ir::mir::downcast::Downcast;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use std::convert::TryFrom;

fn downcast_to_bigint(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok(BigInt256::from(v).into()),
        Value::Short(v) => Ok(BigInt256::from(v).into()),
        Value::Int(v) => Ok(BigInt256::from(v).into()),
        Value::Long(v) => Ok(BigInt256::from(v).into()),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to BigInt",
            in_v
        ))),
    }
}

fn downcast_to_long(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok((v as i64).into()),
        Value::Short(v) => Ok((v as i64).into()),
        Value::Int(v) => Ok((v as i64).into()),
        Value::Long(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to Long",
            in_v
        ))),
    }
}

fn downcast_to_int(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(x) => Ok((x as i32).into()),
        Value::Short(s) => Ok((s as i32).into()),
        Value::Int(_) => Ok(in_v),
        Value::Long(l) => match i32::try_from(l).ok() {
            Some(v) => Ok(v.into()),
            _ => Err(EvalError::UnexpectedValue(
                "Downcast: Int overflow".to_string(),
            )),
        },
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to Int",
            in_v
        ))),
    }
}

fn downcast_to_short(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Short(_) => Ok(in_v),
        Value::Int(i) => match i16::try_from(i).ok() {
            Some(v) => Ok(v.into()),
            _ => Err(EvalError::UnexpectedValue(
                "Downcast: Short overflow".to_string(),
            )),
        },
        Value::Long(l) => match i16::try_from(l).ok() {
            Some(v) => Ok(v.into()),
            _ => Err(EvalError::UnexpectedValue(
                "Downcast: Short overflow".to_string(),
            )),
        },
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to Short",
            in_v
        ))),
    }
}

fn downcast_to_byte(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(_) => Ok(in_v),
        Value::Short(s) => match i8::try_from(s).ok() {
            Some(v) => Ok(v.into()),
            _ => Err(EvalError::UnexpectedValue(
                "Downcast: Byte overflow".to_string(),
            )),
        },
        Value::Int(i) => match i8::try_from(i).ok() {
            Some(v) => Ok(v.into()),
            _ => Err(EvalError::UnexpectedValue(
                "Downcast: Byte overflow".to_string(),
            )),
        },
        Value::Long(l) => match i8::try_from(l).ok() {
            Some(v) => Ok(v.into()),
            _ => Err(EvalError::UnexpectedValue(
                "Downcast: Byte overflow".to_string(),
            )),
        },
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to Byte",
            in_v
        ))),
    }
}

impl Evaluable for Downcast {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match self.tpe {
            SType::SBigInt => downcast_to_bigint(input_v),
            SType::SLong => downcast_to_long(input_v),
            SType::SInt => downcast_to_int(input_v),
            SType::SShort => downcast_to_short(input_v),
            SType::SByte => downcast_to_byte(input_v),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Downcast: expected numeric value, got {0:?}",
                input_v
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergotree_ir::mir::constant::Constant;
    use sigma_test_util::force_any_val;

    use crate::eval::tests::{eval_out_wo_ctx, try_eval_out_wo_ctx};

    use super::*;

    #[test]
    fn to_bigint() {
        let v_byte = force_any_val::<i8>();
        let v_short = force_any_val::<i16>();
        let v_int = force_any_val::<i32>();
        let v_long = force_any_val::<i64>();

        let c_byte: Constant = v_byte.into();
        let c_short: Constant = v_short.into();
        let c_int: Constant = v_int.into();
        let c_long: Constant = v_long.into();

        assert_eq!(
            eval_out_wo_ctx::<BigInt256>(
                &Downcast::new(c_byte.clone().into(), SType::SBigInt)
                    .unwrap()
                    .into()
            ),
            (v_byte as i64).into()
        );
        assert_eq!(
            eval_out_wo_ctx::<BigInt256>(
                &Downcast::new(c_short.clone().into(), SType::SBigInt)
                    .unwrap()
                    .into()
            ),
            (v_short as i64).into()
        );
        assert_eq!(
            eval_out_wo_ctx::<BigInt256>(
                &Downcast::new(c_int.clone().into(), SType::SBigInt)
                    .unwrap()
                    .into()
            ),
            (v_int as i64).into()
        );
        assert_eq!(
            eval_out_wo_ctx::<BigInt256>(
                &Downcast::new(c_long.clone().into(), SType::SBigInt)
                    .unwrap()
                    .into()
            ),
            (v_long as i64).into()
        );
    }

    #[test]
    fn to_long() {
        let v_byte = force_any_val::<i8>();
        let v_short = force_any_val::<i16>();
        let v_int = force_any_val::<i32>();
        let v_long = force_any_val::<i64>();

        let c_byte: Constant = v_byte.into();
        let c_short: Constant = v_short.into();
        let c_int: Constant = v_int.into();
        let c_long: Constant = v_long.into();

        assert_eq!(
            eval_out_wo_ctx::<i64>(
                &Downcast::new(c_byte.clone().into(), SType::SLong)
                    .unwrap()
                    .into()
            ),
            v_byte as i64
        );
        assert_eq!(
            eval_out_wo_ctx::<i64>(
                &Downcast::new(c_short.clone().into(), SType::SLong)
                    .unwrap()
                    .into()
            ),
            v_short as i64
        );
        assert_eq!(
            eval_out_wo_ctx::<i64>(
                &Downcast::new(c_int.clone().into(), SType::SLong)
                    .unwrap()
                    .into()
            ),
            v_int as i64
        );
        assert_eq!(
            eval_out_wo_ctx::<i64>(
                &Downcast::new(c_long.clone().into(), SType::SLong)
                    .unwrap()
                    .into()
            ),
            v_long as i64
        );
    }

    #[test]
    fn to_int() {
        let v_byte = force_any_val::<i8>();
        let v_short = force_any_val::<i16>();
        let v_int = force_any_val::<i32>();
        let v_long = v_int as i64;
        let v_long_oob = if v_long.is_positive() {
            v_long + i32::MAX as i64
        } else {
            v_long - i32::MAX as i64
        };

        let c_byte: Constant = v_byte.into();
        let c_short: Constant = v_short.into();
        let c_int: Constant = v_int.into();
        let c_long: Constant = v_long.into();
        let c_long_oob: Constant = v_long_oob.into();

        assert_eq!(
            eval_out_wo_ctx::<i32>(
                &Downcast::new(c_byte.clone().into(), SType::SInt)
                    .unwrap()
                    .into()
            ),
            v_byte as i32
        );
        assert_eq!(
            eval_out_wo_ctx::<i32>(
                &Downcast::new(c_short.clone().into(), SType::SInt)
                    .unwrap()
                    .into()
            ),
            v_short as i32
        );
        assert_eq!(
            eval_out_wo_ctx::<i32>(
                &Downcast::new(c_int.clone().into(), SType::SInt)
                    .unwrap()
                    .into()
            ),
            v_int as i32
        );
        assert_eq!(
            eval_out_wo_ctx::<i32>(
                &Downcast::new(c_long.clone().into(), SType::SInt)
                    .unwrap()
                    .into()
            ),
            v_long as i32
        );
        assert!(try_eval_out_wo_ctx::<i32>(
            &Downcast::new(c_long_oob.clone().into(), SType::SInt)
                .unwrap()
                .into()
        )
        .is_err())
    }

    #[test]
    fn to_short() {
        let v_short = force_any_val::<i16>();
        let v_int = v_short as i32;
        let v_int_oob = if v_int.is_positive() {
            v_int + i16::MAX as i32
        } else {
            v_int - i16::MAX as i32
        };
        let v_long = v_short as i64;
        let v_long_oob = if v_long.is_positive() {
            v_long + i16::MAX as i64
        } else {
            v_long - i16::MAX as i64
        };

        let c_short: Constant = v_short.into();
        let c_int: Constant = v_int.into();
        let c_int_oob: Constant = v_int_oob.into();
        let c_long: Constant = v_long.into();
        let c_long_oob: Constant = v_long_oob.into();

        assert_eq!(
            eval_out_wo_ctx::<i16>(
                &Downcast::new(c_short.clone().into(), SType::SShort)
                    .unwrap()
                    .into()
            ),
            v_short as i16
        );
        assert_eq!(
            eval_out_wo_ctx::<i16>(
                &Downcast::new(c_int.clone().into(), SType::SShort)
                    .unwrap()
                    .into()
            ),
            v_int as i16
        );
        assert!(try_eval_out_wo_ctx::<i16>(
            &Downcast::new(c_int_oob.clone().into(), SType::SShort)
                .unwrap()
                .into()
        )
        .is_err());

        assert_eq!(
            eval_out_wo_ctx::<i16>(
                &Downcast::new(c_long.clone().into(), SType::SShort)
                    .unwrap()
                    .into()
            ),
            v_long as i16
        );
        assert!(try_eval_out_wo_ctx::<i16>(
            &Downcast::new(c_long_oob.clone().into(), SType::SShort)
                .unwrap()
                .into()
        )
        .is_err());
    }

    #[test]
    fn to_byte() {
        let v_byte = force_any_val::<i8>();
        let v_short = v_byte as i16;
        let v_short_oob = if v_short.is_positive() {
            v_short + i8::MAX as i16
        } else {
            v_short - i8::MAX as i16
        };
        let v_int = v_byte as i32;
        let v_int_oob = if v_int.is_positive() {
            v_int + i8::MAX as i32
        } else {
            v_int - i8::MAX as i32
        };
        let v_long = v_byte as i64;
        let v_long_oob = if v_long.is_positive() {
            v_long + i8::MAX as i64
        } else {
            v_long - i8::MAX as i64
        };

        let c_byte: Constant = v_byte.into();
        let c_short: Constant = v_short.into();
        let c_short_oob: Constant = v_short_oob.into();
        let c_int: Constant = v_int.into();
        let c_int_oob: Constant = v_int_oob.into();
        let c_long: Constant = v_long.into();
        let c_long_oob: Constant = v_long_oob.into();

        assert_eq!(
            eval_out_wo_ctx::<i8>(
                &Downcast::new(c_byte.clone().into(), SType::SByte)
                    .unwrap()
                    .into()
            ),
            v_byte
        );
        assert_eq!(
            eval_out_wo_ctx::<i8>(
                &Downcast::new(c_short.clone().into(), SType::SByte)
                    .unwrap()
                    .into()
            ),
            v_short as i8
        );
        assert!(try_eval_out_wo_ctx::<i8>(
            &Downcast::new(c_short_oob.clone().into(), SType::SByte)
                .unwrap()
                .into()
        )
        .is_err());
        assert_eq!(
            eval_out_wo_ctx::<i8>(
                &Downcast::new(c_int.clone().into(), SType::SByte)
                    .unwrap()
                    .into()
            ),
            v_int as i8
        );
        assert!(try_eval_out_wo_ctx::<i8>(
            &Downcast::new(c_int_oob.clone().into(), SType::SByte)
                .unwrap()
                .into()
        )
        .is_err());
        assert_eq!(
            eval_out_wo_ctx::<i8>(
                &Downcast::new(c_long.clone().into(), SType::SByte)
                    .unwrap()
                    .into()
            ),
            v_long as i8
        );
        assert!(try_eval_out_wo_ctx::<i8>(
            &Downcast::new(c_long_oob.clone().into(), SType::SByte)
                .unwrap()
                .into()
        )
        .is_err());
    }
}
