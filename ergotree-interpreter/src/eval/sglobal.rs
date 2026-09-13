use crate::eval::EvalError;
use alloc::boxed::Box;
use alloc::{string::ToString, sync::Arc};

use ergo_chain_types::autolykos_pow_scheme::{decode_compact_bits, encode_compact_bits};
use ergotree_ir::mir::constant::Literal;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::unsignedbigint256::UnsignedBigInt;
use ergotree_ir::{
    mir::{
        constant::TryExtractInto,
        value::{CollKind, NativeColl, Value},
    },
    serialization::{
        data::DataSerializer,
        sigma_byte_reader::{self, SigmaByteRead},
        sigma_byte_writer::SigmaByteWriter,
    },
};
use num_bigint::BigInt;

use super::EvalFn;
use crate::eval::Vec;
use ergo_chain_types::{autolykos_pow_scheme::AutolykosPowScheme, ec_point::generator};
use ergotree_ir::bigint256::BigInt256;
use ergotree_ir::types::stype::SType;

fn helper_xor(x: &[i8], y: &[i8]) -> Arc<[i8]> {
    x.iter().zip(y.iter()).map(|(x1, x2)| *x1 ^ *x2).collect()
}

pub(crate) static GROUP_GENERATOR_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.groupGenerator expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    Ok(Value::from(generator()))
};

pub(crate) static XOR_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    ctx.add_jit_cost(10)?;
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.xor expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let right_v = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("xor: missing right arg".to_string()))?;
    let left_v = args
        .get(1)
        .cloned()
        .ok_or_else(|| EvalError::NotFound("xor: missing left arg".to_string()))?;

    match (left_v.clone(), right_v.clone()) {
        (
            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(l_byte))),
            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(r_byte))),
        ) => {
            let xor = helper_xor(&l_byte, &r_byte);
            Ok(CollKind::NativeColl(NativeColl::CollByte(xor)).into())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected Xor input to be byte array, got: {0:?}",
            (left_v, right_v)
        ))),
    }
};

pub(crate) static SGLOBAL_FROM_BIGENDIAN_BYTES_EVAL_FN: EvalFn = |mc, _env, ctx, obj, args| {
    ctx.add_jit_cost(10)?;
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.fromBigEndianBytes expected obj to be Value::Global, got {:?}",
            obj
        )));
    }

    let bytes_val = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("fromBigEndianBytes: missing bytes arg".to_string()))?;
    let type_val = mc.tpe().t_range.clone();

    let bytes = match bytes_val {
        Value::Coll(CollKind::NativeColl(NativeColl::CollByte(bytes))) => bytes,
        _ => {
            return Err(EvalError::UnexpectedValue(format!(
                "fromBigEndianBytes: expected first argument to be byte array, got {:?}",
                bytes_val
            )))
        }
    };

    match *type_val {
        SType::SByte => {
            if bytes.len() != 1 {
                return Err(EvalError::UnexpectedValue(
                    "To deserialize Byte with fromBigEndianBytes, exactly one byte should be provided".to_string(),
                ));
            }
            Ok(Value::Byte(bytes[0]))
        }
        SType::SShort => {
            if bytes.len() != 2 {
                return Err(EvalError::UnexpectedValue(
                    "To deserialize Short with fromBigEndianBytes, exactly two bytes should be provided".to_string(),
                ));
            }
            let value = bytes
                .iter()
                .fold(0i16, |acc, &x| (acc << 8) | (x as u8 as i16));
            Ok(Value::Short(value))
        }
        SType::SInt => {
            if bytes.len() != 4 {
                return Err(EvalError::UnexpectedValue(
                    "To deserialize Int with fromBigEndianBytes, exactly four bytes should be provided".to_string(),
                ));
            }
            let value = bytes
                .iter()
                .fold(0i32, |acc, &x| (acc << 8) | (x as u8 as i32));
            Ok(Value::Int(value))
        }
        SType::SLong => {
            if bytes.len() != 8 {
                return Err(EvalError::UnexpectedValue(
                    "To deserialize Long with fromBigEndianBytes, exactly eight bytes should be provided".to_string(),
                ));
            }
            let value = bytes
                .iter()
                .fold(0i64, |acc, &x| (acc << 8) | (x as u8 as i64));
            Ok(Value::Long(value))
        }
        SType::SBigInt => {
            if bytes.len() > 32 {
                return Err(EvalError::UnexpectedValue(
                    "BigInt value doesn't fit into 32 bytes in fromBigEndianBytes".to_string(),
                ));
            }
            let bytes_vec: Vec<u8> = bytes.iter().map(|&x| x as u8).collect();
            Ok(Value::BigInt(
                BigInt256::from_be_slice(&bytes_vec).ok_or_else(|| {
                    EvalError::UnexpectedValue("Failed to convert to BigInt256".to_string())
                })?,
            ))
        }
        SType::SUnsignedBigInt => {
            if bytes.len() > 32 {
                return Err(EvalError::UnexpectedValue(
                    "UnsignedBigInt value doesn't fit into 32 bytes in fromBigEndianBytes"
                        .to_string(),
                ));
            }
            let bytes_vec: Vec<u8> = bytes.iter().map(|&x| x as u8).collect();
            Ok(Value::UnsignedBigInt(
                UnsignedBigInt::from_be_slice(&bytes_vec).ok_or_else(|| {
                    EvalError::UnexpectedValue("Failed to convert to UnsignedBigInt".to_string())
                })?,
            ))
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "Unsupported type provided in fromBigEndianBytes: {:?}",
            type_val
        ))),
    }
};

pub(crate) static DESERIALIZE_EVAL_FN: EvalFn = |mc, _env, ctx, obj, args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.deserialize expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let output_type = &mc.tpe().t_range;
    let bytes = args
        .first()
        .ok_or_else(|| EvalError::NotFound("deserialize: missing first arg".into()))?
        .clone()
        .try_extract_into::<Vec<u8>>()?;
    let n = bytes.len() as u32;
    ctx.add_per_item_jit_cost(100, 32, 32, n)?;
    let mut reader = sigma_byte_reader::from_bytes(&bytes);
    Ok(Value::from(
        reader.with_tree_version(ctx.tree_version(), |reader| {
            DataSerializer::sigma_parse(output_type, reader)
        })?,
    ))
};

pub(crate) static SERIALIZE_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    // Scala `serialize_eval` charges SigmaByteWriter.StartWriterCost = FixedCost(JitCost(10))
    // up front, then each writer `put`'s cost during DataSerializer.serialize.
    ctx.add_jit_cost(10)?;
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.groupGenerator expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let arg: Literal = args
        .first()
        .ok_or_else(|| EvalError::NotFound("serialize: missing first arg".into()))?
        .to_static()
        .try_into()
        .map_err(EvalError::UnexpectedValue)?;

    let mut buf = vec![];
    let put_cost = {
        let mut writer = SigmaByteWriter::new(&mut buf, None);
        writer.enable_serialize_cost_tracking();
        writer.with_tree_version(ctx.tree_version(), |writer| {
            DataSerializer::sigma_serialize(&arg, writer)
        })?;
        writer.serialize_cost()
    };
    ctx.add_jit_cost(put_cost)?;
    Ok(Value::from(buf))
};

pub(crate) static SGLOBAL_SOME_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    ctx.add_jit_cost(5)?;
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.some expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let value = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("some: missing value arg".to_string()))?;
    Ok(Value::Opt(Some(Box::new(value))))
};

pub(crate) static SGLOBAL_NONE_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(5)?;
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.none expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    Ok(Value::Opt(None))
};

pub(crate) static ENCODE_NBITS_EVAL_FN: EvalFn = |_mc, _env, ctx, _obj, args| {
    // Scala Global.encodeNbits EncodeNBitsCost = FixedCost(JitCost(25)).
    ctx.add_jit_cost(25)?;
    let bigint: BigInt = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("encodeNBits: missing first argument".into()))?
        .try_extract_into::<BigInt256>()?
        .into();
    Ok(Value::Long(encode_compact_bits(&bigint)))
};

pub(crate) static DECODE_NBITS_EVAL_FN: EvalFn = |_mc, _env, ctx, _obj, args| {
    // Scala Global.decodeNbits DecodeNBitsCost = FixedCost(JitCost(50)).
    ctx.add_jit_cost(50)?;
    let nbits: i64 = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("decodeNBits: missing first argument".into()))?
        .try_extract_into()?;
    // truncation is safe here, since only bottom 4 bytes are used in decode.
    // nbits is only i64 because Scala doesn't have an unsigned 32-bit type
    Ok(Value::BigInt(
        decode_compact_bits(nbits as u32)
            .try_into()
            .map_err(EvalError::UnexpectedValue)?,
    ))
};
pub(crate) static POW_HIT_EVAL_FN: EvalFn = |_mc, _env, ctx, _obj, mut args| {
    // Pop arguments first: the Scala cost depends on k and the msg/nonce/h lengths.
    let big_n: u32 = args
        .pop()
        .ok_or_else(|| EvalError::NotFound("powHit: missing N".into()))?
        .try_extract_into::<i32>()?
        .try_into()
        .map_err(|_| EvalError::Misc("N out of bounds".into()))?;
    let h = args
        .pop()
        .ok_or_else(|| EvalError::NotFound("powHit: missing h".into()))?
        .try_extract_into::<Vec<u8>>()?;
    let nonce = args
        .pop()
        .ok_or_else(|| EvalError::NotFound("powHit: missing nonce".into()))?
        .try_extract_into::<Vec<u8>>()?;
    let msg = args
        .pop()
        .ok_or_else(|| EvalError::NotFound("powHit: missing msg".into()))?
        .try_extract_into::<Vec<u8>>()?;
    let k = args
        .pop()
        .ok_or_else(|| EvalError::NotFound("powHit: missing msg".into()))?
        .try_extract_into::<i32>()?;
    // Scala PowHitCostKind.cost: baseCost(500) + (k+1) * (totalLen / chunkSize + 1)
    // * perChunkCost, where chunkSize=128 and perChunkCost=7 are CalcBlake2b256's
    // costKind (the heaviest part is k+1 Blake2b256 invocations over msg||nonce||h).
    let total_len = msg.len() + nonce.len() + h.len();
    let chunks = total_len as u64 / 128 + 1;
    let k_plus_1 = u64::try_from(k).map_err(|_| EvalError::Misc("k out of bounds".into()))? + 1;
    ctx.add_jit_cost(500 + k_plus_1 * chunks * 7)?;
    Ok(UnsignedBigInt::try_from(
        AutolykosPowScheme::new(
            k.try_into()
                .map_err(|_| EvalError::Misc("k out of bounds".into()))?,
            big_n,
        )?
        .pow_hit_message_v2(&msg, &nonce, &h, big_n)?,
    )
    .map_err(EvalError::Misc)?
    .into())
};

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergo_chain_types::{
        ADDigest, AutolykosSolution, BlockId, Digest32, EcPoint, Header, Votes,
    };
    use ergotree_ir::bigint256::BigInt256;
    use ergotree_ir::ergo_tree::ErgoTreeVersion;
    use ergotree_ir::mir::avl_tree_data::{AvlTreeData, AvlTreeFlags};
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::long_to_byte_array::LongToByteArray;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
    use ergotree_ir::types::sgroup_elem::GET_ENCODED_METHOD;
    use ergotree_ir::types::stype_param::STypeVar;
    use ergotree_ir::unsignedbigint256::UnsignedBigInt;
    use num_traits::Num;
    use proptest::proptest;

    use crate::eval::test_util::{eval_out, eval_out_wo_ctx, try_eval_out_with_version};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::types::sglobal::{
        self, DECODE_NBITS_METHOD, DESERIALIZE_METHOD, ENCODE_NBITS_METHOD, POW_HIT_METHOD,
        SERIALIZE_METHOD,
    };

    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    fn serialize(val: impl Into<Constant>) -> Vec<u8> {
        let constant = val.into();
        let serialize_node = MethodCall::new(
            Expr::Global,
            SERIALIZE_METHOD.clone().with_concrete_types(
                &[(STypeVar::t(), constant.tpe.clone())]
                    .into_iter()
                    .collect(),
            ),
            vec![constant.into()],
        )
        .unwrap();
        let ctx = force_any_val::<Context>();
        assert!((0u8..ErgoTreeVersion::V3.into()).all(|version| {
            try_eval_out_with_version::<Vec<u8>>(&serialize_node.clone().into(), &ctx, version, 3)
                .is_err()
        }));
        try_eval_out_with_version(&serialize_node.into(), &ctx, ErgoTreeVersion::V3.into(), 3)
            .unwrap()
    }
    fn deserialize(array: &[u8], return_type: SType) -> Constant {
        let type_args = [(STypeVar::t(), return_type)].into_iter().collect();
        let deserialize_node = MethodCall::with_type_args(
            Expr::Global,
            DESERIALIZE_METHOD.clone().with_concrete_types(&type_args),
            vec![Constant::from(array.to_owned()).into()],
            type_args,
        )
        .unwrap();
        let ctx = force_any_val::<Context>();
        assert!((0u8..ErgoTreeVersion::V3.into()).all(|version| {
            try_eval_out_with_version::<Vec<u8>>(&deserialize_node.clone().into(), &ctx, version, 3)
                .is_err()
        }));
        try_eval_out_with_version::<Value>(
            &deserialize_node.into(),
            &ctx,
            ErgoTreeVersion::V3.into(),
            3,
        )
        .unwrap()
        .try_into()
        .unwrap()
    }

    fn encode_nbits(bigint: BigInt256) -> i64 {
        let mc: Expr = MethodCall::new(
            Expr::Global,
            ENCODE_NBITS_METHOD.clone(),
            vec![bigint.into()],
        )
        .unwrap()
        .into();
        eval_out_wo_ctx(&mc)
    }

    fn decode_nbits(nbits: i64) -> BigInt256 {
        let mc: Expr = MethodCall::new(
            Expr::Global,
            DECODE_NBITS_METHOD.clone(),
            vec![nbits.into()],
        )
        .unwrap()
        .into();
        eval_out_wo_ctx(&mc)
    }

    fn create_some_none_method_call<T>(value: Option<T>, tpe: SType) -> Expr
    where
        T: Into<Constant>,
    {
        let type_args = std::iter::once((STypeVar::t(), tpe.clone())).collect();
        match value {
            Some(v) => MethodCall::new(
                Expr::Global,
                sglobal::SOME_METHOD.clone().with_concrete_types(&type_args),
                vec![Expr::Const(v.into())],
            )
            .unwrap()
            .into(),
            None => MethodCall::with_type_args(
                Expr::Global,
                sglobal::NONE_METHOD.clone().with_concrete_types(&type_args),
                vec![],
                type_args,
            )
            .unwrap()
            .into(),
        }
    }

    fn pow_hit(k: u32, msg: &[u8], nonce: &[u8], h: &[u8], big_n: u32) -> UnsignedBigInt {
        let expr: Expr = MethodCall::new(
            Expr::Global,
            POW_HIT_METHOD.clone(),
            vec![
                Constant::from(k as i32).into(),
                Constant::from(msg.to_owned()).into(),
                Constant::from(nonce.to_owned()).into(),
                Constant::from(h.to_owned()).into(),
                Constant::from(big_n as i32).into(),
            ],
        )
        .unwrap()
        .into();
        eval_out_wo_ctx(&expr)
    }

    #[test]
    fn eval_group_generator() {
        let expr: Expr = PropertyCall::new(Expr::Global, sglobal::GROUP_GENERATOR_METHOD.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<EcPoint>(&expr, &ctx),
            ergo_chain_types::ec_point::generator()
        );
    }

    #[test]
    fn eval_xor() {
        let left = vec![1_i8, 1, 0, 0];
        let right = vec![0_i8, 1, 0, 1];
        let expected_xor = vec![1_i8, 0, 0, 1];

        let expr: Expr = MethodCall::new(
            Expr::Global,
            sglobal::XOR_METHOD.clone(),
            vec![right.into(), left.into()],
        )
        .unwrap()
        .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<i8>>(&expr, &ctx).as_slice(),
            expected_xor.as_slice()
        );
    }

    #[test]
    fn test_eval_encode_nbits() {
        assert_eq!(
            encode_nbits(
                BigInt256::from_str_radix("1bc330000000000000000000000000000000000000000000", 16)
                    .unwrap()
            ),
            0x181bc330
        );

        assert_eq!(
            encode_nbits(BigInt256::from_str_radix("12345600", 16).unwrap()),
            0x04123456
        );
        assert_eq!(
            encode_nbits(BigInt256::from_str_radix("-12345600", 16).unwrap()),
            -0x1235
        );
    }

    #[test]
    fn test_eval_decode_nbits() {
        // Following example taken from https://btcinformation.org/en/developer-reference#target-nbits
        let n_bits = 0x181bc330;
        assert_eq!(
            decode_nbits(n_bits),
            BigInt256::from_str_radix("1bc330000000000000000000000000000000000000000000", 16)
                .unwrap()
        );

        let n_bits = 0x01003456;
        assert_eq!(decode_nbits(n_bits), 0x00.into());

        let n_bits = 0x01123456;
        assert_eq!(decode_nbits(n_bits), 0x12.into());

        let n_bits = 0x04923456;
        assert_eq!(decode_nbits(n_bits), (-0x12345600i64).into());

        let n_bits = 0x04123456;
        assert_eq!(decode_nbits(n_bits), 0x12345600.into());

        let n_bits = 0x05123456;
        assert_eq!(decode_nbits(n_bits), 0x1234560000i64.into());

        let n_bits = 16842752;
        assert_eq!(decode_nbits(n_bits), BigInt256::from(1_i8));
    }

    /// Regression: the Global nbits methods charge their Scala costKinds --
    /// Global.encodeNbits = FixedCost(JitCost(25)), Global.decodeNbits =
    /// FixedCost(JitCost(50)) (both were a flat 10, the v6 -15/-40 undercharge).
    /// Isolate each method's costKind by subtracting its arg-const eval cost; the
    /// shared `Global` receiver eval and the MethodCall Fixed(4) cancel, so the
    /// decode-minus-encode costKind delta must be 50 - 25 = 25.
    #[test]
    fn nbits_methods_charge_scala_costkinds() {
        use crate::eval::test_util::eval_out;
        use ergotree_ir::chain::context::Context;
        use ergotree_ir::mir::constant::TryExtractFrom;
        use ergotree_ir::mir::value::Value;
        use sigma_test_util::force_any_val;

        fn cost_of<T: TryExtractFrom<Value<'static>> + 'static>(e: &Expr) -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let _: T = eval_out(e, &ctx);
            ctx.jit_cost_value() - before
        }

        let enc_arg: Expr = Constant::from(BigInt256::from(1i8)).into();
        let dec_arg: Expr = Constant::from(16842752i64).into();
        let enc_mc: Expr = MethodCall::new(
            Expr::Global,
            ENCODE_NBITS_METHOD.clone(),
            vec![enc_arg.clone()],
        )
        .unwrap()
        .into();
        let dec_mc: Expr = MethodCall::new(
            Expr::Global,
            DECODE_NBITS_METHOD.clone(),
            vec![dec_arg.clone()],
        )
        .unwrap()
        .into();

        let enc_kind = cost_of::<i64>(&enc_mc) - cost_of::<BigInt256>(&enc_arg);
        let dec_kind = cost_of::<BigInt256>(&dec_mc) - cost_of::<i64>(&dec_arg);
        assert_eq!(dec_kind - enc_kind, 25);
    }

    /// Regression: `Global.serialize` charges the `SigmaByteWriter` per-`put` costs on top of
    /// `StartWriterCost(10)`. Scala constants (`SigmaByteWriter.scala`): `PutByteCost`=1,
    /// `Put{Signed,Unsigned}NumericCost`=3, `PutChunkCost`=`PerItemCost(3,1,1)` => `3 + n`.
    /// Isolate each value's serialize cost by subtracting its arg-const eval; the shared
    /// `Global` receiver, `MethodCall` Fixed(4) and `StartWriterCost(10)` cancel in the
    /// cross-type differences. Matches the blessed JVM v6 vectors (Byte 90, numerics 92,
    /// Coll[Byte] 95 empty / 98 for 3 bytes).
    #[test]
    fn serialize_charges_writer_costkinds() {
        use crate::eval::test_util::eval_out;
        use ergotree_ir::chain::context::Context;
        use ergotree_ir::mir::constant::TryExtractFrom;
        use ergotree_ir::mir::value::Value;
        use sigma_test_util::force_any_val;

        fn cost_of<T: TryExtractFrom<Value<'static>> + 'static>(e: &Expr) -> u64 {
            let ctx = force_any_val::<Context>();
            // UnsignedBigInt/Option/Header serialize arms are gated to tree version >= V3;
            // pin V3 so eval_out doesn't hit a random pre-V3 context. Cost is version-independent.
            ctx.tree_version.set(ErgoTreeVersion::V3);
            let before = ctx.jit_cost_value();
            let _: T = eval_out(e, &ctx);
            ctx.jit_cost_value() - before
        }

        fn serialize_mc(c: &Constant) -> Expr {
            MethodCall::new(
                Expr::Global,
                SERIALIZE_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), c.tpe.clone())].into_iter().collect()),
                vec![c.clone().into()],
            )
            .unwrap()
            .into()
        }

        // serialize cost of `c` isolated from its argument eval.
        fn ser_kind<T: TryExtractFrom<Value<'static>> + 'static>(c: Constant) -> u64 {
            cost_of::<Vec<u8>>(&serialize_mc(&c)) - cost_of::<T>(&c.into())
        }

        let byte = ser_kind::<i8>(1i8.into());
        let long = ser_kind::<i64>(1i64.into());
        let coll0 = ser_kind::<Vec<i8>>(Vec::<i8>::new().into());
        let coll3 = ser_kind::<Vec<i8>>(vec![1i8, 2, 3].into());
        let bigint = ser_kind::<BigInt256>(BigInt256::from(1i8).into());

        // PutSignedNumericCost(3) - PutByteCost(1)
        assert_eq!(long - byte, 2);
        // Coll[Byte] n=0: putU16(3) + PutChunkCost.cost(0)=3 => 6; minus Byte(1)
        assert_eq!(coll0 - byte, 5);
        // Coll[Byte] n=3: putU16(3) + PutChunkCost.cost(3)=6 => 9; minus Byte(1)
        assert_eq!(coll3 - byte, 8);
        // PutChunkCost per-item slope: cost(3) - cost(0)
        assert_eq!(coll3 - coll0, 3);
        // BigInt(1): putU16(3) + PutChunkCost.cost(1)=4 => 7; minus Byte(1)
        assert_eq!(bigint - byte, 6);

        // --- Phase A1: delegated nested serializers metered at the ergotree-ir site ---
        let gel = ser_kind::<EcPoint>(Constant::from(
            EcPoint::from_base16_str(String::from(
                "026930cb9972e01534918a6f6d6b8e35bc398f57140d13eb3623ea31fbd069939b",
            ))
            .unwrap(),
        ));
        let ubi = ser_kind::<UnsignedBigInt>(Constant::from(UnsignedBigInt::from(1u32)));

        // GroupElement: EcPoint writes one GROUP_SIZE(33)-byte block => PutChunkCost(33)=36; minus Byte(1)
        assert_eq!(gel - byte, 35);
        // UnsignedBigInt(1): putU16(3) + PutChunkCost.cost(1)=4 => 7; identical shape to BigInt(1)
        assert_eq!(ubi, bigint);

        // --- Phase A2c: AvlTree (delegated AvlTreeData serializer) ---
        let avl_dummy = ser_kind::<AvlTreeData>(Constant::from(AvlTreeData {
            digest: ADDigest::zero(),
            tree_flags: AvlTreeFlags::new(true, true, true),
            key_length: 32,
            value_length_opt: None,
        }));
        let avl_withlen = ser_kind::<AvlTreeData>(Constant::from(AvlTreeData {
            digest: ADDigest::zero(),
            tree_flags: AvlTreeFlags::new(true, true, true),
            key_length: 32,
            value_length_opt: Some(Box::new(8u32)),
        }));
        // AvlTree: putBytes(33-digest)=36 + putUByte(flags)=1 + putUInt(keyLength)=0
        // + putOption tag=1 = 38; the Some valueLength inner putUInt nets to 0 (no-info putUInt),
        // so dummy and withValueLen are identical. minus Byte(1) => 37.
        assert_eq!(avl_dummy - byte, 37);
        assert_eq!(avl_withlen - byte, 37);

        // --- Header (delegated Header::scorex_serialize, hand-mirrored at the data.rs site) ---
        // Deterministic v2 / empty-unparsed header (autolykos v2: pk + 8-byte nonce, no w/d).
        let header = ser_kind::<Header>(Constant::from(Header {
            version: 2,
            id: BlockId(Digest32::zero()),
            parent_id: BlockId(Digest32::zero()),
            ad_proofs_root: Digest32::zero(),
            state_root: ADDigest::zero(),
            transaction_root: Digest32::zero(),
            timestamp: 0,
            n_bits: 0,
            height: 0,
            extension_root: Digest32::zero(),
            autolykos_solution: AutolykosSolution {
                miner_pk: Box::new(
                    EcPoint::from_base16_str(String::from(
                        "026930cb9972e01534918a6f6d6b8e35bc398f57140d13eb3623ea31fbd069939b",
                    ))
                    .unwrap(),
                ),
                pow_onetime_pk: None,
                nonce: vec![0u8; 8],
                pow_distance: None,
            },
            votes: Votes([0, 0, 0]),
            unparsed_bytes: Box::new([]),
        }));
        // put_cost 244 (blessed Global.serialize[Header] 333 = 89 + 244): ver 1 + 4×Digest32
        // putBytes(32)=35 + ADDigest putBytes(33)=36 + putULong 3 + nBits putBytes(4)=7
        // + putUInt(height)=0 + votes putBytes(3)=6 + unparsedLen 1 + unparsed putBytes(0)=3
        // + pk putBytes(33)=36 + nonce putBytes(8)=11. minus Byte(1) => 243.
        assert_eq!(header - byte, 243);
    }

    /// The type prefix of each box-register Constant rides through the type serializer under
    /// `Global.serialize[Box]`. A >4-arity tuple register type takes the generic TUPLE
    /// encoding, whose item-count byte the JVM charges (`TypeSerializer.scala:248` `putUByte`
    /// => PutByteCost). Pin the arity boundary: boxes identical but for R4 = 4-tuple vs
    /// 5-tuple of bytes differ by the extra prim type code (1) + the count byte (1) + the
    /// extra data put (1); everything else cancels.
    #[test]
    fn serialize_box_charges_tuple_register_type_count_byte() {
        use crate::eval::test_util::eval_out;
        use ergotree_ir::chain::context::Context;
        use ergotree_ir::chain::ergo_box::box_value::BoxValue;
        use ergotree_ir::chain::ergo_box::{ErgoBox, NonMandatoryRegisters};
        use ergotree_ir::chain::tx_id::TxId;
        use ergotree_ir::ergo_tree::ErgoTree;
        use ergotree_ir::mir::constant::Literal;
        use ergotree_ir::types::stuple::STuple;
        use sigma_test_util::force_any_val;

        fn cost_of(e: &Expr) -> u64 {
            let ctx = force_any_val::<Context>();
            ctx.tree_version.set(ErgoTreeVersion::V3);
            let before = ctx.jit_cost_value();
            let _: Vec<u8> = eval_out(e, &ctx);
            ctx.jit_cost_value() - before
        }

        fn serialize_mc(c: &Constant) -> Expr {
            MethodCall::new(
                Expr::Global,
                SERIALIZE_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), c.tpe.clone())].into_iter().collect()),
                vec![c.clone().into()],
            )
            .unwrap()
            .into()
        }

        fn byte_tuple_const(n: i8) -> Constant {
            let items: Vec<Literal> = (1..=n).map(Literal::from).collect();
            Constant {
                tpe: SType::STuple(STuple {
                    items: vec![SType::SByte; n as usize].try_into().unwrap(),
                }),
                v: Literal::Tup(items.try_into().unwrap()),
            }
        }

        fn box_with_r4(c: Constant) -> Constant {
            let tree = ErgoTree::try_from(Expr::Const(true.into())).unwrap();
            ErgoBox::new(
                BoxValue::SAFE_USER_MIN,
                tree,
                None,
                NonMandatoryRegisters::try_from(vec![c]).unwrap(),
                0,
                TxId::zero(),
                0,
            )
            .unwrap()
            .into()
        }

        let quad = cost_of(&serialize_mc(&box_with_r4(byte_tuple_const(4))));
        let quint = cost_of(&serialize_mc(&box_with_r4(byte_tuple_const(5))));
        // type: TUPLE(1) + count putUByte(1) + 5 prim codes vs PAIR_SYMMETRIC(1) + 4 => +2;
        // data: one more byte put => +1
        assert_eq!(quint - quad, 3);
    }

    /// A legacy box register holding a tuple EXPRESSION (`RegisterValue::ParsedTupleExpr`,
    /// the pre-v5.0 register encoding still on-chain) re-serializes through the expression
    /// serializer; the JVM charges the Tuple op-code byte (`ValueSerializer`'s costed
    /// `put(opCode)`) and the item-count byte (`TupleSerializer`'s `putUByte`), then each
    /// item as a full Constant. Against the same data as a plain Constant register:
    /// tuple-expr (1,2):(Byte,Byte) = opcode(1) + count(1) + 2x[SBYTE code(1) + data put(1)]
    /// = 6 vs the constant register's symmetric-pair type code(1) + 2 data puts = 3.
    #[test]
    fn serialize_box_charges_tuple_expr_register_opcode_and_count() {
        use crate::eval::test_util::eval_out;
        use ergotree_ir::chain::context::Context;
        use ergotree_ir::chain::ergo_box::box_value::BoxValue;
        use ergotree_ir::chain::ergo_box::{
            ErgoBox, EvaluatedTuple, NonMandatoryRegisters, RegisterValue,
        };
        use ergotree_ir::chain::tx_id::TxId;
        use ergotree_ir::ergo_tree::ErgoTree;
        use ergotree_ir::mir::tuple::Tuple;
        use sigma_test_util::force_any_val;

        fn cost_of(e: &Expr) -> u64 {
            let ctx = force_any_val::<Context>();
            ctx.tree_version.set(ErgoTreeVersion::V3);
            let before = ctx.jit_cost_value();
            let _: Vec<u8> = eval_out(e, &ctx);
            ctx.jit_cost_value() - before
        }

        fn serialize_mc(c: &Constant) -> Expr {
            MethodCall::new(
                Expr::Global,
                SERIALIZE_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), c.tpe.clone())].into_iter().collect()),
                vec![c.clone().into()],
            )
            .unwrap()
            .into()
        }

        fn box_with_r4(reg: RegisterValue) -> Constant {
            let tree = ErgoTree::try_from(Expr::Const(true.into())).unwrap();
            ErgoBox::new(
                BoxValue::SAFE_USER_MIN,
                tree,
                None,
                NonMandatoryRegisters::try_from(vec![reg]).unwrap(),
                0,
                TxId::zero(),
                0,
            )
            .unwrap()
            .into()
        }

        let const_reg = RegisterValue::Parsed((1i8, 2i8).into());
        let tuple = Tuple::new(vec![Expr::Const(1i8.into()), Expr::Const(2i8.into())]).unwrap();
        let tup_expr_reg = RegisterValue::ParsedTupleExpr(EvaluatedTuple::new(tuple).unwrap());

        let const_box = cost_of(&serialize_mc(&box_with_r4(const_reg)));
        let tup_expr_box = cost_of(&serialize_mc(&box_with_r4(tup_expr_reg)));
        // opcode(1) + count(1) + per-item SBYTE type codes(2) vs the pair's combined code(1)
        assert_eq!(tup_expr_box - const_box, 3);
    }

    /// `Global.serialize` over a SigmaProp conjecture charges the JVM's child-count (and
    /// Cthreshold's k) `putUShort` => PutUnsignedNumericCost(3) each
    /// (`SigmaBoolean.scala:48/55/61/63`), on top of the per-node op-code byte. With dlog =
    /// opcode(1) + GroupElement chunk(36) = 37: CAND/COR of two dlogs = 1 + 3 + 74 = 78
    /// (so +41 over a single dlog), CTHRESHOLD adds k's 3 more. The shared MethodCall /
    /// StartWriter / arg-const eval costs cancel in the differences.
    #[test]
    fn serialize_charges_sigma_conjecture_child_counts() {
        use crate::eval::test_util::eval_out;
        use ergotree_ir::chain::context::Context;
        use ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
        use ergotree_ir::sigma_protocol::sigma_boolean::cor::Cor;
        use ergotree_ir::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;
        use ergotree_ir::sigma_protocol::sigma_boolean::{ProveDlog, SigmaBoolean};
        use sigma_test_util::force_any_val;

        fn cost_of(e: &Expr) -> u64 {
            let ctx = force_any_val::<Context>();
            ctx.tree_version.set(ErgoTreeVersion::V3);
            let before = ctx.jit_cost_value();
            let _: Vec<u8> = eval_out(e, &ctx);
            ctx.jit_cost_value() - before
        }

        fn serialize_mc(c: &Constant) -> Expr {
            MethodCall::new(
                Expr::Global,
                SERIALIZE_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), c.tpe.clone())].into_iter().collect()),
                vec![c.clone().into()],
            )
            .unwrap()
            .into()
        }

        fn ser_cost(sb: SigmaBoolean) -> u64 {
            cost_of(&serialize_mc(&Constant::from(SigmaProp::new(sb))))
        }

        let dlog = || {
            SigmaBoolean::from(ProveDlog::new(
                EcPoint::from_base16_str(String::from(
                    "026930cb9972e01534918a6f6d6b8e35bc398f57140d13eb3623ea31fbd069939b",
                ))
                .unwrap(),
            ))
        };
        let two_dlogs = || vec![dlog(), dlog()].try_into().unwrap();

        let single = ser_cost(dlog());
        let cand = ser_cost(Cand { items: two_dlogs() }.into());
        let cor = ser_cost(Cor { items: two_dlogs() }.into());
        let cthreshold = ser_cost(
            Cthreshold {
                k: 1,
                children: two_dlogs(),
            }
            .into(),
        );

        // opcode(1) + count putUShort(3) + one more dlog(37)
        assert_eq!(cand - single, 41);
        assert_eq!(cor, cand);
        // k's putUShort(3) on top of CAND's shape
        assert_eq!(cthreshold - cand, 3);
    }

    /// Each box token under `Global.serialize[Box]` is the JVM's `putBytes(id)` =>
    /// PutChunkCost(32) = 35 plus `putULong(amount)` => PutUnsignedNumericCost(3)
    /// (`ErgoBoxCandidate.scala:158/160`; the indexed-digest arm is the unmetered no-info
    /// `putUInt` and never taken here). The token-count byte is written (and charged) for
    /// zero tokens too, so a one-token box costs exactly +38 over a token-less twin.
    #[test]
    fn serialize_box_charges_token_id_and_amount() {
        use crate::eval::test_util::eval_out;
        use ergotree_ir::chain::context::Context;
        use ergotree_ir::chain::ergo_box::box_value::BoxValue;
        use ergotree_ir::chain::ergo_box::{BoxId, BoxTokens, ErgoBox, NonMandatoryRegisters};
        use ergotree_ir::chain::token::{Token, TokenAmount, TokenId};
        use ergotree_ir::chain::tx_id::TxId;
        use ergotree_ir::ergo_tree::ErgoTree;
        use sigma_test_util::force_any_val;

        fn cost_of(e: &Expr) -> u64 {
            let ctx = force_any_val::<Context>();
            ctx.tree_version.set(ErgoTreeVersion::V3);
            let before = ctx.jit_cost_value();
            let _: Vec<u8> = eval_out(e, &ctx);
            ctx.jit_cost_value() - before
        }

        fn serialize_mc(c: &Constant) -> Expr {
            MethodCall::new(
                Expr::Global,
                SERIALIZE_METHOD
                    .clone()
                    .with_concrete_types(&[(STypeVar::t(), c.tpe.clone())].into_iter().collect()),
                vec![c.clone().into()],
            )
            .unwrap()
            .into()
        }

        fn box_with_tokens(tokens: Option<BoxTokens>) -> Constant {
            let tree = ErgoTree::try_from(Expr::Const(true.into())).unwrap();
            ErgoBox::new(
                BoxValue::SAFE_USER_MIN,
                tree,
                tokens,
                NonMandatoryRegisters::empty(),
                0,
                TxId::zero(),
                0,
            )
            .unwrap()
            .into()
        }

        let token = Token::from((
            TokenId::from(BoxId::zero()),
            TokenAmount::try_from(1u64).unwrap(),
        ));
        let without = cost_of(&serialize_mc(&box_with_tokens(None)));
        let with_one = cost_of(&serialize_mc(&box_with_tokens(Some(
            vec![token].try_into().unwrap(),
        ))));
        // id putBytes chunk(3+32) + amount putULong(3)
        assert_eq!(with_one - without, 38);
    }

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn test_bigendian_bytes_roundtrip(
            v_byte in any::<i8>(),
            v_short in any::<i16>(),
            v_int in any::<i32>(),
            v_long in any::<i64>()
        ) {
            {
                let bytes = vec![v_byte];

                let type_args = std::iter::once((STypeVar::t(), SType::SByte)).collect();
                let expr: Expr = MethodCall::with_type_args(
                    Expr::Global,
                    sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                    vec![bytes.into()],
                    type_args,
                )
                .unwrap()
                .into();
                assert_eq!(eval_out_wo_ctx::<i8>(&expr), v_byte);
            }

            {
                let bytes = v_short.to_be_bytes().map(|b| b as i8).to_vec();

                let type_args = std::iter::once((STypeVar::t(), SType::SShort)).collect();
                let expr: Expr = MethodCall::with_type_args(
                    Expr::Global,
                    sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                    vec![bytes.into()],
                    type_args,
                )
                .unwrap()
                .into();
                assert_eq!(eval_out_wo_ctx::<i16>(&expr), v_short);
            }

            {
                let bytes = v_int.to_be_bytes().map(|b| b as i8).to_vec();
                let type_args = std::iter::once((STypeVar::t(), SType::SInt)).collect();
                let expr: Expr = MethodCall::with_type_args(
                    Expr::Global,
                    sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                    vec![bytes.into()],
                    type_args,
                )
                .unwrap()
                .into();
                assert_eq!(eval_out_wo_ctx::<i32>(&expr), v_int);
            }

            {
                let bytes = v_long.to_be_bytes().map(|b| b as i8).to_vec();
                let type_args = std::iter::once((STypeVar::t(), SType::SLong)).collect();
                let expr: Expr = MethodCall::with_type_args(
                    Expr::Global,
                    sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                    vec![bytes.clone().into()],
                    type_args,
                )
                .unwrap()
                .into();
                assert_eq!(eval_out_wo_ctx::<i64>(&expr), v_long);
            }
        }

        #[test]
        fn test_bigint_roundtrip(bigint in any::<BigInt256>()) {
            let bytes = bigint.to_be_bytes().map(|b| b as i8).to_vec();

            let type_args = std::iter::once((STypeVar::t(), SType::SBigInt)).collect();
            let expr: Expr = MethodCall::with_type_args(
                Expr::Global,
                sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                vec![bytes.into()],
                type_args,
            )
            .unwrap()
            .into();
            assert_eq!(eval_out_wo_ctx::<BigInt256>(&expr), bigint);
        }

        #[test]
        fn test_unsigned_bigint_roundtrip(bigint in any::<UnsignedBigInt>()) {
            let bytes = bigint.to_be_bytes().map(|b| b as i8).to_vec();

            let type_args = std::iter::once((STypeVar::t(), SType::SUnsignedBigInt)).collect();
            let expr: Expr = MethodCall::with_type_args(
                Expr::Global,
                sglobal::FROM_BIGENDIAN_BYTES_METHOD.clone().with_concrete_types(&type_args),
                vec![bytes.into()],
                type_args,
            )
            .unwrap()
            .into();
            assert_eq!(eval_out_wo_ctx::<UnsignedBigInt>(&expr), bigint);
        }

        #[test]
        fn test_some_and_none(
            byte_val in any::<i8>(),
            int_val in any::<i32>(),
            long_val in any::<i64>()
        ) {
            assert_eq!(eval_out_wo_ctx::<Option<i8>>(&create_some_none_method_call(Some(byte_val), SType::SByte)), Some(byte_val));
            assert_eq!(eval_out_wo_ctx::<Option<i32>>(&create_some_none_method_call(Some(int_val), SType::SInt)), Some(int_val));
            assert_eq!(eval_out_wo_ctx::<Option<i64>>(&create_some_none_method_call(Some(long_val), SType::SLong)), Some(long_val));
            assert_eq!(eval_out_wo_ctx::<Option<i8>>(&create_some_none_method_call::<i8>(None, SType::SByte)), None);
            assert_eq!(eval_out_wo_ctx::<Option<i64>>(&create_some_none_method_call::<i64>(None, SType::SLong)), None);
        }

    }

    #[test]
    fn serialize_byte() {
        assert_eq!(serialize(-128i8), vec![-128i8 as u8]);
        assert_eq!(serialize(-1i8), vec![-1i8 as u8]);
        assert_eq!(serialize(0i8), vec![0u8]);
        assert_eq!(serialize(1i8), vec![1]);
        assert_eq!(serialize(127i8), vec![127u8]);
    }

    #[test]
    fn serialize_short() {
        assert_eq!(serialize(i16::MIN), vec![0xff, 0xff, 0x03]);
        assert_eq!(serialize(-1i16), vec![0x01]);
        assert_eq!(serialize(0i16), vec![0x00]);
        assert_eq!(serialize(1i16), vec![0x02]);
        assert_eq!(serialize(i16::MAX), vec![0xfe, 0xff, 0x03]);
    }

    #[test]
    fn serialize_byte_array() {
        let arr = vec![0xc0, 0xff, 0xee];
        let serialized = serialize(arr.clone());

        assert_eq!(serialized[0], arr.len() as u8);
        assert_eq!(&serialized[1..], &arr)
    }

    // test that serialize(long) != longToByteArray()
    #[test]
    fn serialize_long_ne_tobytearray() {
        let num = -1000i64;
        let long_to_byte_array = LongToByteArray::try_build(Constant::from(num).into()).unwrap();
        let serialized = serialize(num);
        assert!(serialized != eval_out_wo_ctx::<Vec<u8>>(&long_to_byte_array.into()))
    }

    // test equivalence between Global.serialize and ge.getEncoded
    #[test]
    fn serialize_group_element() {
        let ec_point = EcPoint::from_base16_str(String::from(
            "026930cb9972e01534918a6f6d6b8e35bc398f57140d13eb3623ea31fbd069939b",
        ))
        .unwrap();
        let get_encoded = MethodCall::new(
            Constant::from(ec_point).into(),
            GET_ENCODED_METHOD.clone(),
            vec![],
        )
        .unwrap();
        assert_eq!(
            eval_out_wo_ctx::<Vec<u8>>(&get_encoded.into()),
            serialize(ec_point)
        );
    }

    #[test]
    fn deserialize_group_element() {
        let ec_point = EcPoint::from_base16_str(String::from(
            "026930cb9972e01534918a6f6d6b8e35bc398f57140d13eb3623ea31fbd069939b",
        ))
        .unwrap();
        let get_encoded = MethodCall::new(
            Constant::from(ec_point).into(),
            GET_ENCODED_METHOD.clone(),
            vec![],
        )
        .unwrap();
        let encoded = eval_out_wo_ctx::<Vec<u8>>(&get_encoded.into());
        assert_eq!(
            deserialize(&encoded, SType::SGroupElement),
            Constant::from(ec_point)
        );
    }

    /// Regression: powHit charges the Scala PowHitCostKind formula
    /// `500 + (k+1) * (totalLen/128 + 1) * 7` (was a flat 900). Isolate the
    /// `(k+1)` term with a k-delta: two calls differing only in k by 1 (identical
    /// msg/nonce/h, so totalLen and every other charge cancel) must differ by
    /// `1 * chunks(=1 here) * perChunkCost(=7)` = 7.
    #[test]
    fn pow_hit_charges_scala_costkind() {
        use crate::eval::test_util::try_eval_out;
        use ergotree_ir::chain::context::Context;
        use sigma_test_util::force_any_val;

        let msg = vec![1u8, 2, 3, 4, 5, 6, 7];
        let nonce = vec![0u8; 8];
        let h = vec![0u8; 4]; // totalLen = 19 < 128 -> chunks = 1

        let cost_of = |k: u32| -> u64 {
            let expr: Expr = MethodCall::new(
                Expr::Global,
                POW_HIT_METHOD.clone(),
                vec![
                    Constant::from(k as i32).into(),
                    Constant::from(msg.clone()).into(),
                    Constant::from(nonce.clone()).into(),
                    Constant::from(h.clone()).into(),
                    Constant::from(1024i32 * 1024).into(),
                ],
            )
            .unwrap()
            .into();
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            // cost is charged before the hit computation, so the result is irrelevant
            let _ = try_eval_out::<UnsignedBigInt>(&expr, &ctx);
            ctx.jit_cost_value() - before
        };

        assert_eq!(cost_of(33) - cost_of(32), 7);
    }

    #[test]
    fn pow_hit_eval() {
        let msg = base16::decode("0a101b8c6a4f2e").unwrap();
        let nonce = base16::decode("000000000000002c").unwrap();
        let hbs = base16::decode("00000000").unwrap();
        assert_eq!(
            pow_hit(32, &msg, &nonce, &hbs, 1024 * 1024),
            UnsignedBigInt::from_str_radix(
                "326674862673836209462483453386286740270338859283019276168539876024851191344",
                10
            )
            .unwrap()
        );
    }

    proptest! {
        #[test]
        fn serialize_sigmaprop_eq_prop_bytes(sigma_prop: SigmaProp) {
            let prop_bytes_op = SigmaPropBytes::try_build(Constant::from(sigma_prop.clone()).into()).unwrap();
            let prop_bytes = eval_out_wo_ctx::<Vec<u8>>(&prop_bytes_op.into());
            assert_eq!(serialize(sigma_prop.clone()), &prop_bytes[2..]);
            assert_eq!(deserialize(&prop_bytes[2..], SType::SSigmaProp), sigma_prop.into());
        }
        #[test]
        fn serialize_roundtrip(v in any::<Constant>()) {
            let tpe = v.tpe.clone();
            let res = std::panic::catch_unwind(|| assert_eq!(deserialize(&serialize(v.clone()), tpe.clone()), v));
            if matches!(tpe, SType::SOption(_)) {
                assert!(res.is_err());
            }
            else {
                res.unwrap();
            }
        }
        #[test]
        fn serialize_unsigned_bigint(v in any::<UnsignedBigInt>()) {
            assert_eq!(deserialize(&serialize(v), SType::SUnsignedBigInt), Constant::from(v));
        }
        #[test]
        fn serialize_header(h in any::<Header>()) {
            assert_eq!(deserialize(&serialize(h.clone()), SType::SHeader), Constant::from(h));
        }
    }
}
