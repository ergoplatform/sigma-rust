use alloc::string::ToString;
use ergotree_ir::bigint256::BigInt256;
use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::downcast::Downcast;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;
use ergotree_ir::unsignedbigint256::UnsignedBigInt;
use num_traits::ToPrimitive;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use core::convert::TryFrom;

fn wrap_downcast<T: Into<Value<'static>>>(res: Option<T>) -> Result<Value<'static>, EvalError> {
    res.map(Into::into).ok_or_else(|| {
        EvalError::UnexpectedValue(format!(
            "Downcast: overflow converting to {}",
            core::any::type_name::<T>()
        ))
    })
}

fn downcast_to_bigint<'a>(in_v: Value<'a>, ctx: &Context<'_>) -> Result<Value<'a>, EvalError> {
    match in_v {
        Value::Byte(v) => Ok(BigInt256::from(v).into()),
        Value::Short(v) => Ok(BigInt256::from(v).into()),
        Value::Int(v) => Ok(BigInt256::from(v).into()),
        Value::Long(v) => Ok(BigInt256::from(v).into()),
        Value::BigInt(_) if ctx.tree_version() >= ErgoTreeVersion::V3 => Ok(in_v),
        // Downcasting to BigInt from UnsignedBigInt is intentionally not implemented, use toSigned method instead
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to BigInt",
            in_v
        ))),
    }
}

fn downcast_to_long<'a>(in_v: Value<'a>, ctx: &Context<'_>) -> Result<Value<'a>, EvalError> {
    match in_v {
        Value::Byte(v) => Ok((v as i64).into()),
        Value::Short(v) => Ok((v as i64).into()),
        Value::Int(v) => Ok((v as i64).into()),
        Value::Long(_) => Ok(in_v),
        Value::BigInt(v) if ctx.tree_version() >= ErgoTreeVersion::V3 => wrap_downcast(v.to_i64()),
        Value::UnsignedBigInt(v) if ctx.tree_version() >= ErgoTreeVersion::V3 => {
            wrap_downcast(v.to_i64())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to Long",
            in_v
        ))),
    }
}

fn downcast_to_int<'a>(in_v: Value<'a>, ctx: &Context<'_>) -> Result<Value<'a>, EvalError> {
    match in_v {
        Value::Byte(x) => Ok((x as i32).into()),
        Value::Short(s) => Ok((s as i32).into()),
        Value::Int(_) => Ok(in_v),
        Value::Long(l) => wrap_downcast(l.to_i32()),
        Value::BigInt(v) if ctx.tree_version() >= ErgoTreeVersion::V3 => wrap_downcast(v.to_i32()),
        Value::UnsignedBigInt(v) if ctx.tree_version() >= ErgoTreeVersion::V3 => {
            wrap_downcast(v.to_i32())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to Int",
            in_v
        ))),
    }
}

fn downcast_to_short<'a>(in_v: Value<'a>, ctx: &Context<'_>) -> Result<Value<'a>, EvalError> {
    match in_v {
        Value::Short(_) => Ok(in_v),
        Value::Int(i) => match i16::try_from(i).ok() {
            Some(v) => Ok(v.into()),
            _ => Err(EvalError::UnexpectedValue(
                "Downcast: Short overflow".to_string(),
            )),
        },
        Value::Long(l) => wrap_downcast(l.to_i16()),
        Value::BigInt(v) if ctx.tree_version() >= ErgoTreeVersion::V3 => wrap_downcast(v.to_i16()),
        Value::UnsignedBigInt(v) if ctx.tree_version() >= ErgoTreeVersion::V3 => {
            wrap_downcast(v.to_i16())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to Short",
            in_v
        ))),
    }
}

fn downcast_to_byte<'a>(in_v: Value<'a>, ctx: &Context<'_>) -> Result<Value<'a>, EvalError> {
    match in_v {
        Value::Byte(_) => Ok(in_v),
        Value::Short(s) => wrap_downcast(s.to_i8()),
        Value::Int(i) => wrap_downcast(i.to_i8()),
        Value::Long(l) => wrap_downcast(l.to_i8()),
        Value::BigInt(v) if ctx.tree_version() >= ErgoTreeVersion::V3 => wrap_downcast(v.to_i8()),
        Value::UnsignedBigInt(v) if ctx.tree_version() >= ErgoTreeVersion::V3 => {
            wrap_downcast(v.to_i8())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to Byte",
            in_v
        ))),
    }
}

fn downcast_to_unsigned_bigint<'a>(in_v: Value<'a>) -> Result<Value<'a>, EvalError> {
    // Mirrors Scala `SUnsignedBigInt.downcast` (identical to its upcast): Byte/Short/Int/Long
    // widen, an UnsignedBigInt passes through, and negatives are rejected. No signed-BigInt
    // source arm (use `.toSigned`/`.toUnsigned`). The target-UnsignedBigInt path is
    // unreachable below V3 (the type is V3-gated at parse), so no version guard is needed.
    fn non_negative<'a>(v: i64) -> Result<Value<'a>, EvalError> {
        if v < 0 {
            return Err(EvalError::UnexpectedValue(format!(
                "Downcast: cannot downcast negative value {v} to UnsignedBigInt"
            )));
        }
        Ok(UnsignedBigInt::from(v as u64).into())
    }
    match in_v {
        Value::Byte(v) => non_negative(v as i64),
        Value::Short(v) => non_negative(v as i64),
        Value::Int(v) => non_negative(v as i64),
        Value::Long(v) => non_negative(v),
        Value::UnsignedBigInt(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Downcast: cannot downcast {0:?} to UnsignedBigInt",
            in_v
        ))),
    }
}

impl Evaluable for Downcast {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match self.tpe {
            SType::SBigInt => downcast_to_bigint(input_v, ctx),
            SType::SLong => downcast_to_long(input_v, ctx),
            SType::SInt => downcast_to_int(input_v, ctx),
            SType::SShort => downcast_to_short(input_v, ctx),
            SType::SByte => downcast_to_byte(input_v, ctx),
            SType::SUnsignedBigInt => downcast_to_unsigned_bigint(input_v),
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
    use ergotree_ir::mir::{constant::Constant, expr::Expr};
    use sigma_test_util::force_any_val;

    use crate::eval::test_util::{eval_out_wo_ctx, try_eval_out_with_version, try_eval_out_wo_ctx};

    use super::*;
    use proptest::prelude::*;

    fn downcast(c: impl Into<Constant>, return_type: SType) -> Expr {
        Downcast::new(c.into().into(), return_type).unwrap().into()
    }

    #[test]
    fn downcast_to_unsigned_bigint() {
        assert_eq!(
            eval_out_wo_ctx::<UnsignedBigInt>(&downcast(7i8, SType::SUnsignedBigInt)),
            UnsignedBigInt::from(7u32)
        );
        assert_eq!(
            eval_out_wo_ctx::<UnsignedBigInt>(&downcast(10000i32, SType::SUnsignedBigInt)),
            UnsignedBigInt::from(10000u32)
        );
        assert_eq!(
            eval_out_wo_ctx::<UnsignedBigInt>(&downcast(
                1_000_000_000_000i64,
                SType::SUnsignedBigInt
            )),
            UnsignedBigInt::from(1_000_000_000_000u64)
        );
        // an UnsignedBigInt passes through unchanged
        assert_eq!(
            eval_out_wo_ctx::<UnsignedBigInt>(&downcast(
                UnsignedBigInt::from(42u32),
                SType::SUnsignedBigInt
            )),
            UnsignedBigInt::from(42u32)
        );
    }

    #[test]
    fn downcast_to_unsigned_bigint_rejects_negative_and_signed_bigint() {
        // the JVM errors when converting a negative numeric to UnsignedBigInt
        assert!(
            try_eval_out_wo_ctx::<UnsignedBigInt>(&downcast(-1i32, SType::SUnsignedBigInt))
                .is_err()
        );
        assert!(
            try_eval_out_wo_ctx::<UnsignedBigInt>(&downcast(-1i64, SType::SUnsignedBigInt))
                .is_err()
        );
        // there is no signed-BigInt -> UnsignedBigInt downcast (use .toUnsigned)
        assert!(try_eval_out_wo_ctx::<UnsignedBigInt>(&downcast(
            BigInt256::from(5i64),
            SType::SUnsignedBigInt
        ))
        .is_err());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn to_bigint(v_byte in any::<i8>(), v_short in any::<i16>(), v_int in any::<i32>(), v_long in any::<i64>(), v_bigint in any::<BigInt256>()) {
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(
                    &downcast(v_byte, SType::SBigInt)
                ),
                (v_byte as i64).into()
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(
                    &downcast(v_short, SType::SBigInt)
                ),
                (v_short as i64).into()
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(
                    &downcast(v_int, SType::SBigInt)
                ),
                (v_int as i64).into()
            );
            assert_eq!(
                eval_out_wo_ctx::<BigInt256>(
                    &downcast(v_long, SType::SBigInt)
                ),
                v_long.into()
            );
            let ctx = force_any_val::<Context>();
            (0..ErgoTreeVersion::V3.into())
                .for_each(|version| assert!(try_eval_out_with_version::<BigInt256>(&downcast(v_bigint, SType::SBigInt), &ctx, version, 1).is_err()));
            (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
                |version| {
                    assert_eq!(
                        try_eval_out_with_version::<BigInt256>(
                            &downcast(v_bigint, SType::SBigInt),
                            &ctx,
                            version,
                            version
                        ).unwrap(),
                        v_bigint.clone()
                    )
                },
            );
        }
        #[test]
        fn to_long(v_byte in any::<i8>(), v_short in any::<i16>(), v_int in any::<i32>(), v_long in any::<i64>(), v_bigint in any::<BigInt256>()) {
            let c_byte: Constant = v_byte.into();
            let c_short: Constant = v_short.into();
            let c_int: Constant = v_int.into();
            let c_long: Constant = v_long.into();
            let c_bigint: Constant = v_bigint.into();

            assert_eq!(
                eval_out_wo_ctx::<i64>(&downcast(c_byte, SType::SLong)),
                v_byte as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&downcast(c_short, SType::SLong)),
                v_short as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&downcast(c_int, SType::SLong)),
                v_int as i64
            );
            assert_eq!(
                eval_out_wo_ctx::<i64>(&downcast(c_long, SType::SLong)),
                v_long
            );
            let ctx = force_any_val::<Context>();
            (0..ErgoTreeVersion::V3.into())
                .for_each(|version| assert!(try_eval_out_with_version::<i64>(&downcast(c_bigint.clone(), SType::SLong), &ctx, version, 1).is_err()));
            (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
                |version| {
                    let res = try_eval_out_with_version::<i64>(
                        &downcast(c_bigint.clone(), SType::SLong),
                        &ctx,
                        version,
                        version
                    );
                    if v_bigint < BigInt256::from(i64::MIN) || v_bigint > BigInt256::from(i64::MAX) {
                        assert!(res.is_err());
                    } else {
                        assert_eq!(res.unwrap(), v_bigint.to_i64().unwrap());
                    }
                }
            );
        }
        #[test]
        fn to_int(v_byte in any::<i8>(), v_short in any::<i16>(), v_int in any::<i32>(), v_bigint in any::<BigInt256>()) {
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
                eval_out_wo_ctx::<i32>(&downcast(c_byte, SType::SInt)),
                v_byte as i32
            );
            assert_eq!(
                eval_out_wo_ctx::<i32>(&downcast(c_short, SType::SInt)),
                v_short as i32
            );
            assert_eq!(
                eval_out_wo_ctx::<i32>(&downcast(c_int, SType::SInt)),
                v_int
            );
            assert_eq!(
                eval_out_wo_ctx::<i32>(&downcast(c_long, SType::SInt)),
                v_long as i32
            );
            assert!(try_eval_out_wo_ctx::<i32>(
                &downcast(c_long_oob, SType::SInt)
            )
            .is_err());
            let ctx = force_any_val::<Context>();
            (0..ErgoTreeVersion::V3.into())
                .for_each(|version| assert!(try_eval_out_with_version::<i32>(&downcast(v_bigint, SType::SInt), &ctx, version, 1).is_err()));
            (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
                |version| {
                    let res = try_eval_out_with_version::<i32>(
                        &downcast(v_bigint, SType::SInt),
                        &ctx,
                        version,
                        version
                    );
                    if v_bigint < BigInt256::from(i32::MIN) || v_bigint > BigInt256::from(i32::MAX) {
                        assert!(res.is_err());
                    } else {
                        assert_eq!(res.unwrap(), v_bigint.to_i32().unwrap());
                    }
            });
        }

        #[test]
        fn to_short(v_short in any::<i16>(), v_bigint in any::<BigInt256>()) {
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
                eval_out_wo_ctx::<i16>(&downcast(c_short, SType::SShort)),
                v_short
            );
            assert_eq!(
                eval_out_wo_ctx::<i16>(&downcast(c_int, SType::SShort)),
                v_int as i16
            );
            assert!(try_eval_out_wo_ctx::<i16>(&downcast(c_int_oob, SType::SShort)).is_err());

            assert_eq!(
                eval_out_wo_ctx::<i16>(&downcast(c_long, SType::SShort)),
                v_long as i16
            );
            assert!(try_eval_out_wo_ctx::<i16>(&downcast(c_long_oob, SType::SShort)).is_err());
            let ctx = force_any_val::<Context>();
            (0..ErgoTreeVersion::V3.into())
                .for_each(|version| assert!(try_eval_out_with_version::<i16>(&downcast(v_bigint, SType::SShort), &ctx, version, 1).is_err()));
            (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
                |version| {
                    let res = try_eval_out_with_version::<i16>(
                        &downcast(v_bigint, SType::SShort),
                        &ctx,
                        version,
                        version
                    );
                    if v_bigint < BigInt256::from(i16::MIN) || v_bigint > BigInt256::from(i16::MAX) {
                        assert!(res.is_err());
                    } else {
                        assert_eq!(res.unwrap(), v_bigint.to_i16().unwrap());
                    }
            });
        }
        #[test]
        fn to_byte(v_byte in any::<i8>(), v_bigint in any::<BigInt256>()) {
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
                eval_out_wo_ctx::<i8>(&downcast(c_byte, SType::SByte)),
                v_byte
            );
            assert_eq!(
                eval_out_wo_ctx::<i8>(&downcast(c_short, SType::SByte)),
                v_short as i8
            );
            assert!(try_eval_out_wo_ctx::<i8>(&downcast(c_short_oob, SType::SByte)).is_err());
            assert_eq!(
                eval_out_wo_ctx::<i8>(&downcast(c_int, SType::SByte)),
                v_int as i8
            );
            assert!(try_eval_out_wo_ctx::<i8>(&downcast(c_int_oob, SType::SByte)).is_err());
            assert_eq!(
                eval_out_wo_ctx::<i8>(&downcast(c_long, SType::SByte)),
                v_long as i8
            );
            assert!(try_eval_out_wo_ctx::<i8>(&downcast(c_long_oob, SType::SByte)).is_err());
            let ctx = force_any_val::<Context>();
            (0..ErgoTreeVersion::V3.into())
                .for_each(|version| assert!(try_eval_out_with_version::<i8>(&downcast(v_bigint, SType::SByte), &ctx, version, 1).is_err()));
            (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
                |version| {
                    let res = try_eval_out_with_version::<i8>(
                        &downcast(v_bigint, SType::SByte),
                        &ctx,
                        version,
                        version
                    );
                    if v_bigint < BigInt256::from(i16::MIN) || v_bigint > BigInt256::from(i16::MAX) {
                        assert!(res.is_err());
                    } else {
                        assert_eq!(res.unwrap(), v_bigint.to_i8().unwrap());
                    }
            });
        }
        #[test]
        fn test_overflow(v_short_oob in (i8::MAX as i16 + 1..i16::MAX).prop_union(i16::MIN..i8::MIN as i16),
                         v_int_oob in (i16::MAX as i32 + 1..i32::MAX).prop_union(i32::MIN..i16::MIN as i32),
                         v_long_oob in (i32::MAX as i64 + 1..i64::MAX).prop_union(i64::MIN..i32::MIN as i64)) {
            let v_bigint_oob = BigInt256::from(v_long_oob);
            assert!(try_eval_out_wo_ctx::<i8>(&downcast(v_short_oob, SType::SByte)).is_err());
            assert!(try_eval_out_wo_ctx::<i8>(&downcast(v_int_oob, SType::SByte)).is_err());
            assert!(try_eval_out_wo_ctx::<i8>(&downcast(v_long_oob, SType::SByte)).is_err());
            assert!(try_eval_out_wo_ctx::<i8>(&downcast(v_bigint_oob, SType::SByte)).is_err());
        }
    }
}
