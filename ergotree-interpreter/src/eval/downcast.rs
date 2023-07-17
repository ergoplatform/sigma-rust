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
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
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

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergotree_ir::mir::constant::Constant;

    use crate::eval::tests::{eval_out_wo_ctx, try_eval_out_wo_ctx};

    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn to_bigint(v_byte in any::<i8>(), v_short in any::<i16>(), v_int in any::<i32>(), v_long in any::<i64>()) {
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(
                    &Downcast::new(v_byte.into(), SType::SBigInt).unwrap().into()
                ),
                (v_byte as i64).into()
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(
                    &Downcast::new(v_short.into(), SType::SBigInt)
                        .unwrap()
                        .into()
                ),
                (v_short as i64).into()
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(
                    &Downcast::new(v_int.into(), SType::SBigInt).unwrap().into()
                ),
                (v_int as i64).into()
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(
                    &Downcast::new(v_long.into(), SType::SBigInt).unwrap().into()
                ),
                v_long.into()
            );
        }
        #[test]
        fn to_long(v_byte in any::<i8>(), v_short in any::<i16>(), v_int in any::<i32>(), v_long in any::<i64>()) {
            let c_byte: Constant = v_byte.into();
            let c_short: Constant = v_short.into();
            let c_int: Constant = v_int.into();
            let c_long: Constant = v_long.into();

            assert_eq!(
                eval_out_wo_ctx::<i64>(&Downcast::new(c_byte.into(), SType::SLong).unwrap().into()),
                v_byte as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&Downcast::new(c_short.into(), SType::SLong).unwrap().into()),
                v_short as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&Downcast::new(c_int.into(), SType::SLong).unwrap().into()),
                v_int as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&Downcast::new(c_long.into(), SType::SLong).unwrap().into()),
                v_long
            );
        }
        #[test]
        fn to_int(v_byte in any::<i8>(), v_short in any::<i16>(), v_int in any::<i32>()) {
            let v_long = v_int as i64;
            let v_long_oob = if v_long.is_positive() {
                v_long + i32::MAX as i64 + 1
            } else {
                v_long + i32::MIN as i64 - 1
            };

            let c_byte: Constant = v_byte.into();
            let c_short: Constant = v_short.into();
            let c_int: Constant = v_int.into();
            let c_long: Constant = v_long.into();
            let c_long_oob: Constant = v_long_oob.into();

            assert_eq!(
                eval_out_wo_ctx::<i32>(&Downcast::new(c_byte.into(), SType::SInt).unwrap().into()),
                v_byte as i32
            );
            assert_eq!(
                eval_out_wo_ctx::<i32>(&Downcast::new(c_short.into(), SType::SInt).unwrap().into()),
                v_short as i32
            );
            assert_eq!(
                eval_out_wo_ctx::<i32>(&Downcast::new(c_int.into(), SType::SInt).unwrap().into()),
                v_int
            );
            assert_eq!(
                eval_out_wo_ctx::<i32>(&Downcast::new(c_long.into(), SType::SInt).unwrap().into()),
                v_long as i32
            );
            assert!(try_eval_out_wo_ctx::<i32>(
                &Downcast::new(c_long_oob.into(), SType::SInt)
                    .unwrap()
                    .into()
            )
            .is_err())
        }

        #[test]
        fn to_short(v_short in any::<i16>()) {
            let v_int = v_short as i32;
            let v_int_oob = if v_int.is_positive() {
                v_int + i16::MAX as i32 + 1
            } else {
                v_int + i16::MIN as i32 - 1
            };
            let v_long = v_short as i64;
            let v_long_oob = if v_long.is_positive() {
                v_long + i16::MAX as i64
            } else {
                v_long + i16::MIN as i64 - 1
            };

            let c_short: Constant = v_short.into();
            let c_int: Constant = v_int.into();
            let c_int_oob: Constant = v_int_oob.into();
            let c_long: Constant = v_long.into();
            let c_long_oob: Constant = v_long_oob.into();

            assert_eq!(
                eval_out_wo_ctx::<i16>(&Downcast::new(c_short.into(), SType::SShort).unwrap().into()),
                v_short
            );
            assert_eq!(
                eval_out_wo_ctx::<i16>(&Downcast::new(c_int.into(), SType::SShort).unwrap().into()),
                v_int as i16
            );
            assert!(try_eval_out_wo_ctx::<i16>(
                &Downcast::new(c_int_oob.into(), SType::SShort)
                    .unwrap()
                    .into()
            )
            .is_err());

            assert_eq!(
                eval_out_wo_ctx::<i16>(&Downcast::new(c_long.into(), SType::SShort).unwrap().into()),
                v_long as i16
            );
            assert!(try_eval_out_wo_ctx::<i16>(
                &Downcast::new(c_long_oob.into(), SType::SShort)
                    .unwrap()
                    .into()
            )
            .is_err());
        }
        #[test]
        fn to_byte(v_byte in any::<i8>()) {
            let v_short = v_byte as i16;
            let v_short_oob = if v_short.is_positive() {
                v_short + i8::MAX as i16 + 1
            } else {
                v_short + i8::MIN as i16 - 1
            };
            let v_int = v_byte as i32;
            let v_int_oob = if v_int.is_positive() {
                v_int + i8::MAX as i32
            } else {
                v_int + i8::MIN as i32 - 1
            };
            let v_long = v_byte as i64;
            let v_long_oob = if v_long.is_positive() {
                v_long + i8::MAX as i64
            } else {
                v_long + i8::MIN as i64 - 1
            };

            let c_byte: Constant = v_byte.into();
            let c_short: Constant = v_short.into();
            let c_short_oob: Constant = v_short_oob.into();
            let c_int: Constant = v_int.into();
            let c_int_oob: Constant = v_int_oob.into();
            let c_long: Constant = v_long.into();
            let c_long_oob: Constant = v_long_oob.into();

            assert_eq!(
                eval_out_wo_ctx::<i8>(&Downcast::new(c_byte.into(), SType::SByte).unwrap().into()),
                v_byte
            );
            assert_eq!(
                eval_out_wo_ctx::<i8>(&Downcast::new(c_short.into(), SType::SByte).unwrap().into()),
                v_short as i8
            );
            assert!(try_eval_out_wo_ctx::<i8>(
                &Downcast::new(c_short_oob.into(), SType::SByte)
                .unwrap()
                .into()
            )
            .is_err());
            assert_eq!(
                eval_out_wo_ctx::<i8>(&Downcast::new(c_int.into(), SType::SByte).unwrap().into()),
                v_int as i8
            );
            assert!(try_eval_out_wo_ctx::<i8>(
                &Downcast::new(c_int_oob.into(), SType::SByte)
                    .unwrap()
                    .into()
            )
            .is_err());
            assert_eq!(
                eval_out_wo_ctx::<i8>(&Downcast::new(c_long.into(), SType::SByte).unwrap().into()),
                v_long as i8
            );
            assert!(try_eval_out_wo_ctx::<i8>(
                &Downcast::new(c_long_oob.into(), SType::SByte)
                    .unwrap()
                    .into()
            )
            .is_err());
        }
        #[test]
        fn test_overflow(v_short_oob in (i8::MAX as i16 + 1..i16::MAX).prop_union(i16::MIN..i8::MIN as i16),
                         v_int_oob in (i16::MAX as i32 + 1..i32::MAX).prop_union(i32::MIN..i16::MIN as i32),
                         v_long_oob in (i32::MAX as i64 + 1..i64::MAX).prop_union(i64::MIN..i32::MIN as i64)) {
            let c_short_oob: Constant = v_short_oob.into();
            let c_int_oob: Constant = v_int_oob.into();
            let c_long_oob: Constant = v_long_oob.into();
            assert!(try_eval_out_wo_ctx::<i8>(
                &Downcast::new(c_short_oob.into(), SType::SByte)
                .unwrap()
                .into()
            )
            .is_err());
            assert!(try_eval_out_wo_ctx::<i8>(
                &Downcast::new(c_int_oob.clone().into(), SType::SByte)
                .unwrap()
                .into()
            )
            .is_err());
            assert!(try_eval_out_wo_ctx::<i8>(
                &Downcast::new(c_long_oob.clone().into(), SType::SByte)
                .unwrap()
                .into()
            )
            .is_err());

            assert!(try_eval_out_wo_ctx::<i16>(
                &Downcast::new(c_int_oob.into(), SType::SByte)
                .unwrap()
                .into()
            )
            .is_err());
            assert!(try_eval_out_wo_ctx::<i16>(
                &Downcast::new(c_long_oob.clone().into(), SType::SByte)
                .unwrap()
                .into()
            )
            .is_err());
            assert!(try_eval_out_wo_ctx::<i32>(
                &Downcast::new(c_long_oob.into(), SType::SByte)
                .unwrap()
                .into()
            )
            .is_err());
        }
    }
}
