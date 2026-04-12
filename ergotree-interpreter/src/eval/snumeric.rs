use alloc::vec::Vec;
use ergotree_ir::{
    bigint256::BigInt256,
    mir::{constant::TryExtractInto, value::Value},
    types::{
        smethod::SMethod,
        snumeric::{
            self,
            sbigint::{TO_UNSIGNED_METHOD_ID, TO_UNSIGNED_MOD_METHOD_ID},
            sunsignedbigint::{
                MOD_INVERSE_METHOD_ID, MOD_METHOD_ID, MULTIPLY_MOD_METHOD_ID, PLUS_MOD_METHOD_ID,
                SUBTRACT_MOD_METHOD_ID, TO_SIGNED_METHOD_ID,
            },
        },
        stype_companion::STypeCompanion,
    },
    unsignedbigint256::UnsignedBigInt,
};
use num_traits::{CheckedRem, CheckedShl, CheckedShr};

use super::cost_accum::add_fixed_cost;
use super::costs;
use super::{EvalError, EvalFn};

const TO_BYTES_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    add_fixed_cost(ctx, costs::SNUMERIC_TO_BYTES_COST)?;
    Ok(match obj {
        Value::Byte(obj) => obj.to_be_bytes().to_vec().into(),
        Value::Short(obj) => obj.to_be_bytes().to_vec().into(),
        Value::Int(obj) => obj.to_be_bytes().to_vec().into(),
        Value::Long(obj) => obj.to_be_bytes().to_vec().into(),
        Value::BigInt(obj) => obj.to_be_bytes().to_vec().into(),
        Value::UnsignedBigInt(obj) => obj.to_be_bytes().to_vec().into(),
        other => {
            return Err(EvalError::UnexpectedValue(format!(
                "Expected numeric type, got {other:?}"
            )))
        }
    })
};

static TO_BITS_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    add_fixed_cost(ctx, costs::SNUMERIC_TO_BITS_COST)?;
    fn byte_to_bits(mut byte: u8) -> [bool; 8] {
        let mut res = [false; 8];
        let mut i = 8;
        while byte != 0 {
            i -= 1;
            res[i] = (byte & 1) == 1;
            byte >>= 1;
        }
        res
    }
    fn to_bits(bytes: &[u8]) -> Value<'static> {
        bytes
            .iter()
            .copied()
            .flat_map(byte_to_bits)
            .collect::<Vec<_>>()
            .into()
    }
    Ok(match obj {
        Value::Byte(obj) => to_bits(obj.to_be_bytes().as_slice()),
        Value::Short(obj) => to_bits(obj.to_be_bytes().as_slice()),
        Value::Int(obj) => to_bits(obj.to_be_bytes().as_slice()),
        Value::Long(obj) => to_bits(obj.to_be_bytes().as_slice()),
        Value::BigInt(obj) => to_bits(obj.to_be_bytes().as_slice()),
        Value::UnsignedBigInt(obj) => to_bits(obj.to_be_bytes().as_slice()),
        other => {
            return Err(EvalError::UnexpectedValue(format!(
                "Expected numeric type, got {other:?}"
            )))
        }
    })
};

static BITWISE_INVERSE_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    add_fixed_cost(ctx, costs::SNUMERIC_BITWISE_INVERSE_COST)?;
    Ok(match obj {
        Value::Byte(obj) => (!obj).into(),
        Value::Short(obj) => (!obj).into(),
        Value::Int(obj) => (!obj).into(),
        Value::Long(obj) => (!obj).into(),
        Value::BigInt(obj) => (!obj).into(),
        Value::UnsignedBigInt(obj) => (!obj).into(),
        other => {
            return Err(EvalError::UnexpectedValue(format!(
                "Expected numeric type, got {other:?}"
            )))
        }
    })
};

static BITWISE_OR_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_BITWISE_OP_COST)?;
    let rhs = args[0].clone();
    Ok(match _mc.obj_type {
        STypeCompanion::SByte => {
            (obj.try_extract_into::<i8>()? | rhs.try_extract_into::<i8>()?).into()
        }
        STypeCompanion::SShort => {
            (obj.try_extract_into::<i16>()? | rhs.try_extract_into::<i16>()?).into()
        }
        STypeCompanion::SInt => {
            (obj.try_extract_into::<i32>()? | rhs.try_extract_into::<i32>()?).into()
        }
        STypeCompanion::SLong => {
            (obj.try_extract_into::<i64>()? | rhs.try_extract_into::<i64>()?).into()
        }
        STypeCompanion::SBigInt => {
            (obj.try_extract_into::<BigInt256>()? | rhs.try_extract_into::<BigInt256>()?).into()
        }
        STypeCompanion::SUnsignedBigInt => (obj.try_extract_into::<UnsignedBigInt>()?
            | rhs.try_extract_into::<UnsignedBigInt>()?)
        .into(),
        other => {
            return Err(EvalError::UnexpectedValue(format!(
                "Expected numeric type, got {other:?}"
            )))
        }
    })
};

static BITWISE_AND_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_BITWISE_OP_COST)?;
    let rhs = args[0].clone();
    Ok(match _mc.obj_type {
        STypeCompanion::SByte => {
            (obj.try_extract_into::<i8>()? & rhs.try_extract_into::<i8>()?).into()
        }
        STypeCompanion::SShort => {
            (obj.try_extract_into::<i16>()? & rhs.try_extract_into::<i16>()?).into()
        }
        STypeCompanion::SInt => {
            (obj.try_extract_into::<i32>()? & rhs.try_extract_into::<i32>()?).into()
        }
        STypeCompanion::SLong => {
            (obj.try_extract_into::<i64>()? & rhs.try_extract_into::<i64>()?).into()
        }
        STypeCompanion::SBigInt => {
            (obj.try_extract_into::<BigInt256>()? & rhs.try_extract_into::<BigInt256>()?).into()
        }
        STypeCompanion::SUnsignedBigInt => (obj.try_extract_into::<UnsignedBigInt>()?
            & rhs.try_extract_into::<UnsignedBigInt>()?)
        .into(),
        other => {
            return Err(EvalError::UnexpectedValue(format!(
                "Expected numeric type, got {other:?}"
            )))
        }
    })
};

static BITWISE_XOR_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_BITWISE_OP_COST)?;
    let rhs = args
        .first()
        .ok_or_else(|| EvalError::UnexpectedValue("rhs missing".into()))?
        .clone();
    Ok(match _mc.obj_type {
        STypeCompanion::SByte => {
            (obj.try_extract_into::<i8>()? ^ rhs.try_extract_into::<i8>()?).into()
        }
        STypeCompanion::SShort => {
            (obj.try_extract_into::<i16>()? ^ rhs.try_extract_into::<i16>()?).into()
        }
        STypeCompanion::SInt => {
            (obj.try_extract_into::<i32>()? ^ rhs.try_extract_into::<i32>()?).into()
        }
        STypeCompanion::SLong => {
            (obj.try_extract_into::<i64>()? ^ rhs.try_extract_into::<i64>()?).into()
        }
        STypeCompanion::SBigInt => {
            (obj.try_extract_into::<BigInt256>()? ^ rhs.try_extract_into::<BigInt256>()?).into()
        }
        STypeCompanion::SUnsignedBigInt => (obj.try_extract_into::<UnsignedBigInt>()?
            ^ rhs.try_extract_into::<UnsignedBigInt>()?)
        .into(),
        other => {
            return Err(EvalError::UnexpectedValue(format!(
                "Expected numeric type, got {other:?}"
            )))
        }
    })
};

#[inline(never)]
fn invalid_shift_err() -> EvalError {
    EvalError::Misc("shift value is out of bounds".into())
}

static SHIFT_LEFT_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_SHIFT_COST)?;
    let shift_value: u32 = args
        .first()
        .ok_or_else(|| EvalError::UnexpectedValue("shift arg missing".into()))?
        .clone()
        .try_extract_into::<i32>()?
        .try_into()
        .map_err(|_| EvalError::UnexpectedValue("expected non-negative shift value".into()))?;
    Ok(match obj {
        Value::Byte(obj) => obj
            .checked_shl(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::Short(obj) => obj
            .checked_shl(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::Int(obj) => obj
            .checked_shl(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::Long(obj) => obj
            .checked_shl(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::BigInt(obj) => obj
            .checked_shl(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::UnsignedBigInt(obj) => obj
            .checked_shl(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        other => {
            return Err(EvalError::UnexpectedValue(format!(
                "Expected numeric type, got {other:?}"
            )))
        }
    })
};

static SHIFT_RIGHT_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_SHIFT_COST)?;
    let shift_value: u32 = args
        .first()
        .ok_or_else(|| EvalError::UnexpectedValue("shift arg missing".into()))?
        .clone()
        .try_extract_into::<i32>()?
        .try_into()
        .map_err(|_| EvalError::UnexpectedValue("expected non-negative shift value".into()))?;
    Ok(match obj {
        Value::Byte(obj) => obj
            .checked_shr(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::Short(obj) => obj
            .checked_shr(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::Int(obj) => obj
            .checked_shr(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::Long(obj) => obj
            .checked_shr(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::BigInt(obj) => obj
            .checked_shr(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        Value::UnsignedBigInt(obj) => obj
            .checked_shr(shift_value)
            .ok_or_else(invalid_shift_err)?
            .into(),
        other => {
            return Err(EvalError::UnexpectedValue(format!(
                "Expected numeric type, got {other:?}"
            )))
        }
    })
};

static TO_UNSIGNED_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    add_fixed_cost(ctx, costs::SNUMERIC_TO_UNSIGNED_COST)?;
    let signed = obj.try_extract_into::<BigInt256>()?;
    UnsignedBigInt::try_from(signed)
        .map_err(|err| EvalError::ArithmeticException(err.into()))
        .map(Value::from)
};

static TO_UNSIGNED_MOD_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_TO_UNSIGNED_MOD_COST)?;
    let signed = obj.try_extract_into::<BigInt256>()?;
    let modulus = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("toUnsignedMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    UnsignedBigInt::from_signed_mod(signed, modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("toUnsignedMod: can't divide by 0".into()))
};

static MOD_INVERSE_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_MOD_INVERSE_COST)?;
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("modInv: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.mod_inv(modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("modInv: can't divide by 0".into()))
};

static PLUS_MOD_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_PLUS_MOD_COST)?;
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let b = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("plusMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("plusMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.checked_mod_add(b, modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("plusMod: can't divide by 0".into()))
};

static SUBTRACT_MOD_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_SUBTRACT_MOD_COST)?;
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let b = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("subtractMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("subtractMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.checked_mod_sub(b, modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("subtractMod: can't divide by 0".into()))
};

static MULTIPLY_MOD_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_MULTIPLY_MOD_COST)?;
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let b = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("multiplyMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("multiplyMod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.checked_mod_mul(b, modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("multiplyMod: can't divide by 0".into()))
};

static MOD_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    add_fixed_cost(ctx, costs::SNUMERIC_MOD_COST)?;
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    let modulus = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::UnexpectedValue("mod: missing first argument".into()))?
        .try_extract_into::<UnsignedBigInt>()?;
    obj.checked_rem(&modulus)
        .map(Value::from)
        .ok_or_else(|| EvalError::ArithmeticException("mod: can't divide by 0".into()))
};

static TO_SIGNED_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    add_fixed_cost(ctx, costs::SNUMERIC_TO_SIGNED_COST)?;
    let obj = obj.try_extract_into::<UnsignedBigInt>()?;
    BigInt256::try_from(obj)
        .map_err(|e| EvalError::ArithmeticException(e.into()))
        .map(Value::from)
};

// List of methods that are available for all numeric types
fn snumeric_evalfn(method: &SMethod) -> Result<EvalFn, EvalError> {
    match method.method_id() {
        // The following methods are explicitly not supported
        snumeric::TO_BYTE_METHOD_ID
        | snumeric::TO_INT_METHOD_ID
        | snumeric::TO_SHORT_METHOD_ID
        | snumeric::TO_LONG_METHOD_ID
        | snumeric::TO_BIGINT_METHOD_ID => Err(EvalError::NotFound("Not implemented".into())),
        snumeric::TO_BYTES_METHOD_ID => Ok(TO_BYTES_EVAL_FN),
        snumeric::TO_BITS_METHOD_ID => Ok(TO_BITS_EVAL_FN),
        snumeric::BITWISE_INVERSE_METHOD_ID => Ok(BITWISE_INVERSE_EVAL_FN),
        snumeric::BITWISE_OR_METHOD_ID => Ok(BITWISE_OR_EVAL_FN),
        snumeric::BITWISE_AND_METHOD_ID => Ok(BITWISE_AND_EVAL_FN),
        snumeric::BITWISE_XOR_METHOD_ID => Ok(BITWISE_XOR_EVAL_FN),
        snumeric::SHIFT_LEFT_METHOD_ID => Ok(SHIFT_LEFT_EVAL_FN),
        snumeric::SHIFT_RIGHT_METHOD_ID => Ok(SHIFT_RIGHT_EVAL_FN),
        _ => Err(EvalError::NotFound(format!(
            "Method {:?} not found",
            method.method_id()
        ))),
    }
}

fn bigint_evalfn(method: &SMethod) -> Result<EvalFn, EvalError> {
    match snumeric_evalfn(method) {
        Ok(eval_fn) => Ok(eval_fn),
        Err(_) => match method.method_id() {
            TO_UNSIGNED_METHOD_ID => Ok(TO_UNSIGNED_EVAL_FN),
            TO_UNSIGNED_MOD_METHOD_ID => Ok(TO_UNSIGNED_MOD_EVAL_FN),
            _ => Err(EvalError::NotFound(format!(
                "SBigInt: Method id {:?} not found",
                method.method_id()
            ))),
        },
    }
}

fn unsigned_bigint_evalfn(method: &SMethod) -> Result<EvalFn, EvalError> {
    match snumeric_evalfn(method) {
        Ok(eval_fn) => Ok(eval_fn),
        Err(_) => match method.method_id() {
            MOD_INVERSE_METHOD_ID => Ok(MOD_INVERSE_EVAL_FN),
            PLUS_MOD_METHOD_ID => Ok(PLUS_MOD_EVAL_FN),
            SUBTRACT_MOD_METHOD_ID => Ok(SUBTRACT_MOD_EVAL_FN),
            MULTIPLY_MOD_METHOD_ID => Ok(MULTIPLY_MOD_EVAL_FN),
            MOD_METHOD_ID => Ok(MOD_EVAL_FN),
            TO_SIGNED_METHOD_ID => Ok(TO_SIGNED_EVAL_FN),
            _ => Err(EvalError::NotFound(format!(
                "SUnsignedBigInt: Method id {:?} not found",
                method.method_id()
            ))),
        },
    }
}

pub(crate) fn numeric_method_evalfn(method: &SMethod) -> Result<EvalFn, EvalError> {
    match method.obj_type.type_code() {
        snumeric::sbyte::TYPE_CODE
        | snumeric::sshort::TYPE_CODE
        | snumeric::sint::TYPE_CODE
        | snumeric::slong::TYPE_CODE => snumeric_evalfn(method),
        snumeric::sbigint::TYPE_CODE => bigint_evalfn(method),
        snumeric::sunsignedbigint::TYPE_CODE => unsigned_bigint_evalfn(method),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Expected numeric type, found {:?}",
            method.obj_type.type_code()
        ))),
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod test {
    use core::{iter::Sum, ops::Not};

    use ergotree_ir::{
        bigint256::BigInt256,
        mir::{
            constant::{Constant, TryExtractFrom},
            expr::Expr,
            method_call::MethodCall,
            value::Value,
        },
        types::{
            sglobal,
            smethod::{SMethod, SMethodDesc},
            snumeric::{
                self,
                sbigint::{TO_UNSIGNED_METHOD_DESC, TO_UNSIGNED_MOD_METHOD_DESC},
                sunsignedbigint::{
                    MOD_INVERSE_METHOD_DESC, MOD_METHOD_DESC, MULTIPLY_MOD_METHOD_DESC,
                    PLUS_MOD_METHOD_DESC, SUBTRACT_MOD_METHOD_DESC, TO_SIGNED_METHOD_DESC,
                },
                BITWISE_AND_METHOD_ID, BITWISE_INVERSE_METHOD_ID, BITWISE_OR_METHOD_ID,
                BITWISE_XOR_METHOD_ID, SHIFT_LEFT_METHOD_ID, SHIFT_RIGHT_METHOD_ID,
                TO_BITS_METHOD_ID, TO_BYTES_METHOD_ID,
            },
            stype::LiftIntoSType,
            stype_companion::STypeCompanion,
            stype_param::STypeVar,
        },
        unsignedbigint256::UnsignedBigInt,
    };
    use num_traits::{CheckedRem, CheckedShl, CheckedShr, One, Zero};
    use proptest::prelude::*;

    use crate::eval::{
        test_util::{eval_out_wo_ctx, try_eval_out_wo_ctx},
        EvalError,
    };

    fn eval_modular_op(
        desc: &SMethodDesc,
        args: &[UnsignedBigInt],
    ) -> Result<UnsignedBigInt, EvalError> {
        let mc: Expr = MethodCall::new(
            Constant::from(args[0]).into(),
            SMethod::new(STypeCompanion::SUnsignedBigInt, desc.clone()),
            args[1..]
                .iter()
                .copied()
                .map(Constant::from)
                .map(Expr::from)
                .collect(),
        )
        .unwrap()
        .into();
        try_eval_out_wo_ctx(&mc)
    }
    trait Numeric:
        LiftIntoSType
        + TryExtractFrom<Value<'static>>
        + PartialEq
        + Into<Constant>
        + core::fmt::Debug
        + Clone
        + 'static
        + One
        + Zero
        + CheckedShl
        + CheckedShr
        + Not<Output = Self>
        + Sum
    {
    }
    impl Numeric for i8 {}
    impl Numeric for i16 {}
    impl Numeric for i32 {}
    impl Numeric for i64 {}
    impl Numeric for BigInt256 {}
    impl Numeric for UnsignedBigInt {}
    fn big_endian_roundtrip<T: Numeric>(v: T, methods: &[SMethod]) {
        let type_args = std::iter::once((STypeVar::t(), T::stype())).collect();
        let to_be_bytes_expr: Expr = MethodCall::new(
            <T as Into<Constant>>::into(v.clone()).into(),
            methods
                .iter()
                .find(|method| method.method_id() == TO_BYTES_METHOD_ID)
                .unwrap()
                .clone(),
            vec![],
        )
        .unwrap()
        .into();
        let expr: Expr = MethodCall::with_type_args(
            Expr::Global,
            sglobal::FROM_BIGENDIAN_BYTES_METHOD
                .clone()
                .with_concrete_types(&type_args),
            vec![to_be_bytes_expr],
            type_args,
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<T>(&expr), v);
    }
    fn bits_roundtrip<T: Numeric>(v: T, methods: &[SMethod]) {
        let to_bits: Expr = MethodCall::new(
            <T as Into<Constant>>::into(v.clone()).into(),
            methods
                .iter()
                .find(|method| method.method_id() == TO_BITS_METHOD_ID)
                .unwrap()
                .clone(),
            vec![],
        )
        .unwrap()
        .into();
        let res: Vec<bool> = eval_out_wo_ctx(&to_bits);
        // perform binary multiplication to make sure bits equals long
        let res = res
            .into_iter()
            .rev()
            .map(|bit| if bit { T::one() } else { T::zero() })
            .enumerate()
            .map(|(i, bit)| bit << i as u32)
            .sum::<T>();

        assert_eq!(res, v);
    }

    fn bitwise_inverse_test<T: Numeric>(v: T, methods: &[SMethod]) {
        let inverse_mc: Expr = MethodCall::new(
            <T as Into<Constant>>::into(v.clone()).into(),
            methods
                .iter()
                .find(|method| method.method_id() == BITWISE_INVERSE_METHOD_ID)
                .unwrap()
                .clone(),
            vec![],
        )
        .unwrap()
        .into();
        let res: T = eval_out_wo_ctx(&inverse_mc);
        assert_eq!(res, !v);
    }

    fn bitwise_or<T: Numeric>(v: T, rhs: T, methods: &[SMethod]) -> T {
        let or_mc: Expr = MethodCall::new(
            <T as Into<Constant>>::into(v.clone()).into(),
            methods
                .iter()
                .find(|method| method.method_id() == BITWISE_OR_METHOD_ID)
                .unwrap()
                .clone(),
            vec![<T as Into<Constant>>::into(rhs).into()],
        )
        .unwrap()
        .into();
        eval_out_wo_ctx(&or_mc)
    }

    fn bitwise_and<T: Numeric>(v: T, rhs: T, methods: &[SMethod]) -> T {
        let and_mc: Expr = MethodCall::new(
            <T as Into<Constant>>::into(v.clone()).into(),
            methods
                .iter()
                .find(|method| method.method_id() == BITWISE_AND_METHOD_ID)
                .unwrap()
                .clone(),
            vec![<T as Into<Constant>>::into(rhs).into()],
        )
        .unwrap()
        .into();
        eval_out_wo_ctx(&and_mc)
    }
    fn bitwise_xor<T: Numeric>(v: T, rhs: T, methods: &[SMethod]) -> T {
        let xor_mc: Expr = MethodCall::new(
            <T as Into<Constant>>::into(v.clone()).into(),
            methods
                .iter()
                .find(|method| method.method_id() == BITWISE_XOR_METHOD_ID)
                .unwrap()
                .clone(),
            vec![<T as Into<Constant>>::into(rhs).into()],
        )
        .unwrap()
        .into();
        eval_out_wo_ctx(&xor_mc)
    }
    fn shl<T: Numeric>(v: T, rhs: u32, methods: &[SMethod]) -> T {
        let shl_mc: Expr = MethodCall::new(
            <T as Into<Constant>>::into(v.clone()).into(),
            methods
                .iter()
                .find(|method| method.method_id() == SHIFT_LEFT_METHOD_ID)
                .unwrap()
                .clone(),
            vec![Constant::from(rhs as i32).into()],
        )
        .unwrap()
        .into();
        eval_out_wo_ctx(&shl_mc)
    }
    fn shr<T: Numeric>(v: T, rhs: u32, methods: &[SMethod]) -> T {
        let shr_mc: Expr = MethodCall::new(
            <T as Into<Constant>>::into(v.clone()).into(),
            methods
                .iter()
                .find(|method| method.method_id() == SHIFT_RIGHT_METHOD_ID)
                .unwrap()
                .clone(),
            vec![Constant::from(rhs as i32).into()],
        )
        .unwrap()
        .into();
        eval_out_wo_ctx(&shr_mc)
    }
    #[test]
    fn bitwise_or_byte() {
        assert_eq!(bitwise_or(127i8, -128i8, &snumeric::sbyte::METHODS), -1i8);
    }
    proptest! {
        #[test]
        fn byte_big_endian_roundtrip(byte in any::<i8>()) {
            big_endian_roundtrip(byte, &snumeric::sbyte::METHODS);
        }
        #[test]
        fn short_big_endian_roundtrip(short in any::<i16>()) {
            big_endian_roundtrip(short, &snumeric::sshort::METHODS);
        }
        #[test]
        fn int_big_endian_roundtrip(int in any::<i32>()) {
            big_endian_roundtrip(int, &snumeric::sint::METHODS);
        }
        #[test]
        fn long_big_endian_roundtrip(long in any::<i64>()) {
            big_endian_roundtrip(long, &snumeric::slong::METHODS);
        }
        #[test]
        fn bigint_big_endian_roundtrip(bigint in any::<BigInt256>()) {
            big_endian_roundtrip(bigint, &snumeric::sbigint::METHODS);
        }
        #[test]
        fn unsigned_bigint_big_endian_roundtrip(bigint in any::<UnsignedBigInt>()) {
            big_endian_roundtrip(bigint, &snumeric::sunsignedbigint::METHODS);
        }
        #[test]
        fn to_bits_byte(byte in any::<i8>()) {
            bits_roundtrip(byte, &snumeric::sbyte::METHODS);
        }
        #[test]
        fn to_bits_short(short in any::<i16>()) {
            bits_roundtrip(short, &snumeric::sshort::METHODS);
        }
        #[test]
        fn to_bits_int(int in any::<i32>()) {
            bits_roundtrip(int, &snumeric::sint::METHODS);
        }
        #[test]
        fn to_bits_long(long in any::<i64>()) {
            bits_roundtrip(long, &snumeric::slong::METHODS);
        }
        #[test]
        fn to_bits_bigint(bigint in any::<BigInt256>()) {
            bits_roundtrip(bigint, &snumeric::sbigint::METHODS);
        }
        #[test]
        fn to_bits_unsigned_bigint(bigint in any::<UnsignedBigInt>()) {
            bits_roundtrip(bigint, &snumeric::sunsignedbigint::METHODS);
        }
        #[test]
        fn inverse_byte(b in any::<i8>()) {
            bitwise_inverse_test(b, &snumeric::sbyte::METHODS);
        }
        #[test]
        fn inverse_short(short in any::<i16>()) {
            bitwise_inverse_test(short, &snumeric::sshort::METHODS);
        }
        #[test]
        fn inverse_int(int in any::<i32>()) {
            bitwise_inverse_test(int, &snumeric::sint::METHODS);
        }
        #[test]
        fn inverse_long(long in any::<i64>()) {
            bitwise_inverse_test(long, &snumeric::slong::METHODS);
        }
        #[test]
        fn inverse_bigint(bigint in any::<BigInt256>()) {
            bitwise_inverse_test(bigint, &snumeric::sbigint::METHODS);
        }
        #[test]
        fn inverse_unsigned_bigint(bigint in any::<UnsignedBigInt>()) {
            bitwise_inverse_test(bigint, &snumeric::sunsignedbigint::METHODS);
        }

        #[test]
        fn bitwise_or_byte_arbitrary(a in any::<i8>(), b in any::<i8>()) {
            assert_eq!(bitwise_or(a, b, &snumeric::sbyte::METHODS), a | b);
        }
        #[test]
        fn bitwise_or_short_arbitrary(a in any::<i16>(), b in any::<i16>()) {
            assert_eq!(bitwise_or(a, b, &snumeric::sshort::METHODS), a | b);
        }
        #[test]
        fn bitwise_or_int_arbitrary(a in any::<i32>(), b in any::<i32>()) {
            assert_eq!(bitwise_or(a, b, &snumeric::sint::METHODS), a | b);
        }
        #[test]
        fn bitwise_or_long_arbitrary(a in any::<i64>(), b in any::<i64>()) {
            assert_eq!(bitwise_or(a, b, &snumeric::slong::METHODS), a | b);
        }
        #[test]
        fn bitwise_or_bigint_arbitrary(a in any::<BigInt256>(), b in any::<BigInt256>()) {
            assert_eq!(bitwise_or(a, b, &snumeric::sbigint::METHODS), a | b);
        }
        #[test]
        fn bitwise_or_unsigned_bigint_arbitrary(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>()) {
            assert_eq!(bitwise_or(a, b, &snumeric::sunsignedbigint::METHODS), a | b);
        }
        #[test]
        fn bitwise_and_byte_arbitrary(a in any::<i8>(), b in any::<i8>()) {
            assert_eq!(bitwise_and(a, b, &snumeric::sbyte::METHODS), a & b);
        }
        #[test]
        fn bitwise_and_short_arbitrary(a in any::<i16>(), b in any::<i16>()) {
            assert_eq!(bitwise_and(a, b, &snumeric::sshort::METHODS), a & b);
        }
        #[test]
        fn bitwise_and_int_arbitrary(a in any::<i32>(), b in any::<i32>()) {
            assert_eq!(bitwise_and(a, b, &snumeric::sint::METHODS), a & b);
        }
        #[test]
        fn bitwise_and_long_arbitrary(a in any::<i64>(), b in any::<i64>()) {
            assert_eq!(bitwise_and(a, b, &snumeric::slong::METHODS), a & b);
        }
        #[test]
        fn bitwise_and_bigint_arbitrary(a in any::<BigInt256>(), b in any::<BigInt256>()) {
            assert_eq!(bitwise_and(a, b, &snumeric::sbigint::METHODS), a & b);
        }
        #[test]
        fn bitwise_and_unsigned_bigint_arbitrary(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>()) {
            assert_eq!(bitwise_and(a, b, &snumeric::sunsignedbigint::METHODS), a & b);
        }
        #[test]
        fn bitwise_xor_byte_arbitrary(a in any::<i8>(), b in any::<i8>()) {
            assert_eq!(bitwise_xor(a, b, &snumeric::sbyte::METHODS), a ^ b);
        }
        #[test]
        fn bitwise_xor_short_arbitrary(a in any::<i16>(), b in any::<i16>()) {
            assert_eq!(bitwise_xor(a, b, &snumeric::sshort::METHODS), a ^ b);
        }
        #[test]
        fn bitwise_xor_int_arbitrary(a in any::<i32>(), b in any::<i32>()) {
            assert_eq!(bitwise_xor(a, b, &snumeric::sint::METHODS), a ^ b);
        }
        #[test]
        fn bitwise_xor_long_arbitrary(a in any::<i64>(), b in any::<i64>()) {
            assert_eq!(bitwise_xor(a, b, &snumeric::slong::METHODS), a ^ b);
        }
        #[test]
        fn bitwise_xor_bigint_arbitrary(a in any::<BigInt256>(), b in any::<BigInt256>()) {
            assert_eq!(bitwise_xor(a, b, &snumeric::sbigint::METHODS), a ^ b);
        }
        #[test]
        fn bitwise_xor_unsigned_bigint_arbitrary(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>()) {
            assert_eq!(bitwise_xor(a, b, &snumeric::sunsignedbigint::METHODS), a ^ b);
        }
        #[test]
        fn shl_byte_arbitrary(a in any::<i8>(), shift in 0u32..8) {
            assert_eq!(shl(a, shift, &snumeric::sbyte::METHODS), a << shift);
        }
        #[test]
        fn shl_short_arbitrary(a in any::<i16>(), shift in 0u32..16) {
            assert_eq!(shl(a, shift, &snumeric::sshort::METHODS), a << shift);
        }
        #[test]
        fn shl_int_arbitrary(a in any::<i32>(), shift in 0u32..32) {
            assert_eq!(shl(a, shift, &snumeric::sint::METHODS), a << shift);
        }
        #[test]
        fn shl_long_arbitrary(a in any::<i64>(), shift in 0u32..64) {
            assert_eq!(shl(a, shift, &snumeric::slong::METHODS), a << shift);
        }
        #[test]
        fn shl_bigint_arbitrary(a in any::<BigInt256>(), shift in 0u32..256) {
            assert_eq!(shl(a, shift, &snumeric::sbigint::METHODS), a << shift);
        }
        #[test]
        fn shl_unsigned_bigint_arbitrary(a in any::<UnsignedBigInt>(), shift in 0u32..256) {
            assert_eq!(shl(a, shift, &snumeric::sunsignedbigint::METHODS), a << shift);
        }
        #[test]
        #[should_panic]
        fn shl_byte_arbitrary_invalid(a in any::<i8>(), shift in 8u32..) {
            assert_eq!(shl(a, shift, &snumeric::sbyte::METHODS), a << shift);
        }
        #[test]
        #[should_panic]
        fn shl_short_arbitrary_invalid(a in any::<i16>(), shift in 16u32..) {
            assert_eq!(shl(a, shift, &snumeric::sshort::METHODS), a << shift);
        }
        #[test]
        #[should_panic]
        fn shl_int_arbitrary_invalid(a in any::<i32>(), shift in 32u32..) {
            assert_eq!(shl(a, shift, &snumeric::sint::METHODS), a << shift);
        }
        #[test]
        #[should_panic]
        fn shl_long_arbitrary_invalid(a in any::<i64>(), shift in 64u32..) {
            assert_eq!(shl(a, shift, &snumeric::slong::METHODS), a << shift);
        }
        #[test]
        #[should_panic]
        fn shl_bigint_arbitrary_invalid(a in any::<BigInt256>(), shift in 256u32..) {
            assert_eq!(shl(a, shift, &snumeric::sbigint::METHODS), a << shift);
        }
        #[test]
        #[should_panic]
        fn shl_unsigned_bigint_arbitrary_invalid(a in any::<UnsignedBigInt>(), shift in 256u32..) {
            assert_eq!(shl(a, shift, &snumeric::sunsignedbigint::METHODS), a << shift);
        }
        #[test]
        fn shr_byte_arbitrary(a in any::<i8>(), shift in 0u32..8) {
            assert_eq!(shr(a, shift, &snumeric::sbyte::METHODS), a >> shift);
        }
        #[test]
        fn shr_short_arbitrary(a in any::<i16>(), shift in 0u32..16) {
            assert_eq!(shr(a, shift, &snumeric::sshort::METHODS), a >> shift);
        }
        #[test]
        fn shr_int_arbitrary(a in any::<i32>(), shift in 0u32..32) {
            assert_eq!(shr(a, shift, &snumeric::sint::METHODS), a >> shift);
        }
        #[test]
        fn shr_long_arbitrary(a in any::<i64>(), shift in 0u32..64) {
            assert_eq!(shr(a, shift, &snumeric::slong::METHODS), a >> shift);
        }
        #[test]
        fn shr_bigint_arbitrary(a in any::<BigInt256>(), shift in 0u32..256) {
            assert_eq!(shr(a, shift, &snumeric::sbigint::METHODS), a >> shift);
        }
        #[test]
        fn shr_unsigned_bigint_arbitrary(a in any::<UnsignedBigInt>(), shift in 0u32..256) {
            assert_eq!(shr(a, shift, &snumeric::sunsignedbigint::METHODS), a >> shift);
        }
        #[test]
        #[should_panic]
        fn shr_byte_arbitrary_invalid(a in any::<i8>(), shift in 8u32..) {
            assert_eq!(shr(a, shift, &snumeric::sbyte::METHODS), a >> shift);
        }
        #[test]
        #[should_panic]
        fn shr_short_arbitrary_invalid(a in any::<i16>(), shift in 16u32..) {
            assert_eq!(shr(a, shift, &snumeric::sshort::METHODS), a >> shift);
        }
        #[test]
        #[should_panic]
        fn shr_int_arbitrary_invalid(a in any::<i32>(), shift in 32u32..) {
            assert_eq!(shr(a, shift, &snumeric::sint::METHODS), a >> shift);
        }
        #[test]
        #[should_panic]
        fn shr_long_arbitrary_invalid(a in any::<i64>(), shift in 64u32..) {
            assert_eq!(shr(a, shift, &snumeric::slong::METHODS), a >> shift);
        }
        #[test]
        #[should_panic]
        fn shr_bigint_arbitrary_invalid(a in any::<BigInt256>(), shift in 256u32..) {
            assert_eq!(shr(a, shift, &snumeric::sbigint::METHODS), a >> shift);
        }
        #[test]
        #[should_panic]
        fn shr_unsigned_bigint_arbitrary_invalid(a in any::<UnsignedBigInt>(), shift in 256u32..) {
            assert_eq!(shr(a, shift, &snumeric::sunsignedbigint::METHODS), a >> shift);
        }

        #[test]
        fn eval_to_unsigned(signed in any::<BigInt256>()) {
            let mc: Expr = MethodCall::new(
                Constant::from(signed).into(),
                SMethod::new(STypeCompanion::SBigInt, TO_UNSIGNED_METHOD_DESC.clone()),
                vec![],
            )
            .unwrap()
            .into();
            let res = try_eval_out_wo_ctx::<UnsignedBigInt>(&mc);
            if signed < 0.into() {
                assert!(res.is_err())
            } else {
                assert_eq!(res.unwrap().to_string(), signed.to_string());
            }
        }
        #[test]
        fn eval_to_unsigned_mod(signed in any::<BigInt256>(), modulus in any::<UnsignedBigInt>()) {
            let mc: Expr = MethodCall::new(
                Constant::from(signed).into(),
                SMethod::new(STypeCompanion::SBigInt, TO_UNSIGNED_MOD_METHOD_DESC.clone()),
                vec![Constant::from(modulus).into()],
            )
            .unwrap()
            .into();
            assert_eq!(try_eval_out_wo_ctx::<UnsignedBigInt>(&mc).ok(), UnsignedBigInt::from_signed_mod(signed, modulus));
        }

        #[test]
        fn eval_to_signed(unsigned in any::<UnsignedBigInt>()) {
            let mc: Expr = MethodCall::new(
                Constant::from(unsigned).into(),
                SMethod::new(STypeCompanion::SUnsignedBigInt, TO_SIGNED_METHOD_DESC.clone()),
                vec![],
            )
            .unwrap()
            .into();
            let res = try_eval_out_wo_ctx::<BigInt256>(&mc);
            assert_eq!(res.ok(), BigInt256::try_from(unsigned).ok());
        }

        #[test]
        fn eval_mod_ops(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>(), modulus in any::<UnsignedBigInt>()) {
            assert_eq!(eval_modular_op(&MOD_METHOD_DESC, &[a, modulus]).ok(), a.checked_rem(&modulus));
            assert_eq!(eval_modular_op(&PLUS_MOD_METHOD_DESC, &[a, b, modulus]).ok(), a.checked_mod_add(b, modulus));
            assert_eq!(eval_modular_op(&SUBTRACT_MOD_METHOD_DESC, &[a, b, modulus]).ok(), a.checked_mod_sub(b, modulus));
            assert_eq!(eval_modular_op(&MULTIPLY_MOD_METHOD_DESC, &[a, b, modulus]).ok(), a.checked_mod_mul(b, modulus));
            assert_eq!(eval_modular_op(&MOD_INVERSE_METHOD_DESC, &[a, modulus]).ok(), a.mod_inv(modulus));
        }
    }
}
