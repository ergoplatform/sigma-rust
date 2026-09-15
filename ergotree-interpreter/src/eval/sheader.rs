//! Evaluating predefined `Header` (or SHeader) type properties

use alloc::sync::Arc;
use core::convert::TryInto;
use num_bigint::ToBigInt;

use alloc::vec::Vec;
use ergo_chain_types::Header;
use ergotree_ir::{
    bigint256::BigInt256,
    mir::{constant::TryExtractInto, value::Value},
};

use super::{EvalError, EvalFn};

pub(crate) static VERSION_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.version as i8).into())
};

pub(crate) static ID_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.id).into())
};

pub(crate) static PARENT_ID_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.parent_id).into())
};

pub(crate) static AD_PROOFS_ROOT_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.ad_proofs_root).into())
};

pub(crate) static STATE_ROOT_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.state_root).into())
};

pub(crate) static TRANSACTION_ROOT_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.transaction_root).into())
};

pub(crate) static EXTENSION_ROOT_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.extension_root).into())
};

pub(crate) static TIMESTAMP_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.timestamp as i64).into())
};

pub(crate) static N_BITS_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.n_bits as i64).into())
};

pub(crate) static HEIGHT_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.height as i32).into())
};

pub(crate) static MINER_PK_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(Arc::new(*header.autolykos_solution.miner_pk).into())
};

pub(crate) static POW_ONETIME_PK_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok((*header.autolykos_solution.pow_onetime_pk.unwrap_or_default()).into())
};

pub(crate) static POW_NONCE_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(header.autolykos_solution.nonce.into())
};

pub(crate) static POW_DISTANCE_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    #[allow(clippy::unwrap_used)] // unsigned->signed conversion never fails
    let pow_distance: BigInt256 = header
        .autolykos_solution
        .pow_distance
        .unwrap_or_default()
        .to_bigint()
        .unwrap()
        .try_into()
        .map_err(EvalError::Misc)?;
    Ok(pow_distance.into())
};

pub(crate) static VOTES_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(10)?;
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<u8>>::into(header.votes).into())
};

pub(crate) static CHECK_POW_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(700)?;
    match obj {
        Value::Header(header) => Ok(header.check_pow()?.into()),
        _ => Err(EvalError::UnexpectedValue(format!(
            "SHeader.checkpow expected obj to be Value::Global, got {:?}",
            obj
        ))),
    }
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use core::convert::{TryFrom, TryInto};

    use alloc::{boxed::Box, vec::Vec};
    use ergo_chain_types::{BlockId, Digest, Digest32, EcPoint, Votes};
    use ergotree_ir::{
        bigint256::BigInt256,
        chain::context::Context,
        mir::{
            coll_by_index::ByIndex, expr::Expr, method_call::MethodCall,
            property_call::PropertyCall,
        },
        types::{
            scontext::{self, HEADERS_PROPERTY},
            sheader,
            smethod::SMethod,
        },
    };
    use num_bigint::ToBigInt;
    use sigma_test_util::force_any_val;
    use sigma_util::AsVecU8;

    use crate::eval::test_util::{eval_out, try_eval_out_wo_ctx};

    // Index in Context.headers array
    const HEADER_INDEX: usize = 0;

    // Evaluates `Header.minerPk`, `Header.powOnetimePk`
    fn eval_header_pks(ctx: &Context<'static>) -> [Box<EcPoint>; 2] {
        let miner_pk = eval_out::<EcPoint>(
            &create_get_header_property_expr(sheader::MINER_PK_PROPERTY.clone()),
            ctx,
        );
        let pow_onetime_pk = eval_out::<EcPoint>(
            &create_get_header_property_expr(sheader::POW_ONETIME_PK_PROPERTY.clone()),
            ctx,
        );
        [miner_pk, pow_onetime_pk].map(Box::new)
    }

    // Evaluates `Header.AdProofsRoot`, `Header.transactionRoot`, `Header.extensionRoot`
    fn eval_header_roots(ctx: &Context<'static>) -> [Digest32; 3] {
        vec![
            sheader::AD_PROOFS_ROOT_PROPERTY.clone(),
            sheader::TRANSACTIONS_ROOT_PROPERTY.clone(),
            sheader::EXTENSION_ROOT_PROPERTY.clone(),
        ]
        .into_iter()
        .map(|smethod| eval_out::<Vec<i8>>(&create_get_header_property_expr(smethod), ctx))
        .map(digest_from_bytes_signed::<32>)
        .collect::<Vec<_>>()
        .try_into()
        .expect("internal error: smethods vector length is not equal to 3")
    }

    // Evaluates `Header.id` and `Header.parentId`
    fn eval_header_ids(ctx: &Context<'static>) -> [BlockId; 2] {
        let id = eval_out::<Vec<i8>>(
            &create_get_header_property_expr(sheader::ID_PROPERTY.clone()),
            ctx,
        );
        let parent_id = eval_out::<Vec<i8>>(
            &create_get_header_property_expr(sheader::PARENT_ID_PROPERTY.clone()),
            ctx,
        );
        [id, parent_id].map(block_id_from_bytes_signed)
    }

    fn create_get_header_property_expr(method: SMethod) -> Expr {
        let get_headers_expr = create_get_header_by_index_expr();
        create_header_property_call_expr(get_headers_expr, method)
    }

    // An `Expr` for such code in ErgoScript `CONTEXT.headers(0)`
    fn create_get_header_by_index_expr() -> Expr {
        let prop_call = PropertyCall::new(Expr::Context, scontext::HEADERS_PROPERTY.clone())
            .expect("internal error: invalid headers property call of Context")
            .into();
        ByIndex::new(prop_call, Expr::Const((HEADER_INDEX as i32).into()), None)
            .expect("internal error: invalid types of ByIndex expression")
            .into()
    }

    fn create_header_property_call_expr(headers_expr: Expr, method: SMethod) -> Expr {
        PropertyCall::new(headers_expr, method)
            .expect("internal error: invalid header property call")
            .into()
    }

    fn block_id_from_bytes_signed(bytes: Vec<i8>) -> BlockId {
        let arr32 = digest_from_bytes_signed::<32>(bytes);
        BlockId(arr32)
    }

    fn digest_from_bytes_signed<const N: usize>(bytes: Vec<i8>) -> Digest<N> {
        let arr = arr_from_bytes_signed::<N>(bytes);
        arr.into()
    }

    fn arr_from_bytes_signed<const N: usize>(bytes: Vec<i8>) -> [u8; N] {
        bytes
            .as_vec_u8()
            .try_into()
            .unwrap_or_else(|_| panic!("internal error: bytes buffer length is not equal to {}", N))
    }

    #[test]
    fn test_eval_version() {
        let expr = create_get_header_property_expr(sheader::VERSION_PROPERTY.clone());
        let ctx = force_any_val::<Context>();
        let version = ctx.headers[HEADER_INDEX].version as i8;
        assert_eq!(version, eval_out::<i8>(&expr, &ctx));
    }

    #[test]
    fn test_eval_ids() {
        let ctx = force_any_val::<Context>();
        let expected = ctx
            .headers
            .get(HEADER_INDEX)
            .map(|h| [h.id, h.parent_id])
            .expect("internal error: empty headers array");
        let actual = eval_header_ids(&ctx);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_roots() {
        let ctx = force_any_val::<Context>();
        let expected = ctx
            .headers
            .get(HEADER_INDEX)
            .map(|h| [h.ad_proofs_root, h.transaction_root, h.extension_root])
            .expect("internal error: empty headers array");
        let actual = eval_header_roots(&ctx);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_state_root() {
        let expr = create_get_header_property_expr(sheader::STATE_ROOT_PROPERTY.clone());
        let ctx = force_any_val::<Context>();
        let expected = ctx.headers[HEADER_INDEX].state_root;
        let actual = digest_from_bytes_signed::<33>(eval_out::<Vec<i8>>(&expr, &ctx));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_timestamp() {
        let expr = create_get_header_property_expr(sheader::TIMESTAMP_PROPERTY.clone());
        let ctx = force_any_val::<Context>();
        let expected = ctx.headers[HEADER_INDEX].timestamp as i64;
        let actual = eval_out::<i64>(&expr, &ctx);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_n_bits() {
        let expr = create_get_header_property_expr(sheader::N_BITS_PROPERTY.clone());
        let ctx = force_any_val::<Context>();
        let expected = ctx.headers[HEADER_INDEX].n_bits as i64;
        let actual = eval_out::<i64>(&expr, &ctx);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_height() {
        let expr = create_get_header_property_expr(sheader::HEIGHT_PROPERTY.clone());
        let ctx = force_any_val::<Context>();
        let expected = ctx.headers[HEADER_INDEX].height as i32;
        let actual = eval_out::<i32>(&expr, &ctx);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_pks() {
        let ctx = force_any_val::<Context>();
        let expected = ctx
            .headers
            .get(HEADER_INDEX)
            .map(|h| {
                [
                    h.autolykos_solution.miner_pk.clone(),
                    h.autolykos_solution
                        .pow_onetime_pk
                        .clone()
                        .unwrap_or_default(),
                ]
            })
            .expect("internal error: empty headers array");
        let actual = eval_header_pks(&ctx);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_pow_distance() {
        let expr = create_get_header_property_expr(sheader::POW_DISTANCE_PROPERTY.clone());
        let ctx = force_any_val::<Context>();
        let expected = ctx.headers[HEADER_INDEX]
            .autolykos_solution
            .pow_distance
            .clone()
            .unwrap_or_default()
            .to_bigint()
            .unwrap();
        let actual = {
            let bi = eval_out::<BigInt256>(&expr, &ctx);
            bi.into()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_pow_nonce() {
        let expr = create_get_header_property_expr(sheader::POW_NONCE_PROPERTY.clone());
        let ctx = force_any_val::<Context>();
        let expected = ctx.headers[HEADER_INDEX].autolykos_solution.nonce.clone();
        let actual = eval_out::<Vec<i8>>(&expr, &ctx).as_vec_u8();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_votes() {
        let expr = create_get_header_property_expr(sheader::VOTES_PROPERTY.clone());
        let ctx = force_any_val::<Context>();
        let expected = ctx.headers[HEADER_INDEX].votes.clone();
        let actual = {
            let votes_bytes = eval_out::<Vec<i8>>(&expr, &ctx).as_vec_u8();
            Votes::try_from(votes_bytes)
                .expect("internal error: votes bytes buffer length isn't equal to 3")
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_failed_invalid_obj() {
        // calling for Header property on Context obj
        let expr: Expr = PropertyCall {
            obj: Box::new(Expr::Context),
            method: sheader::VERSION_PROPERTY.clone(),
        }
        .into();
        assert!(try_eval_out_wo_ctx::<i8>(&expr).is_err());
    }

    #[test]
    fn test_eval_failed_unknown_property() {
        let unknown_property = {
            use ergotree_ir::types::{
                smethod::{MethodId, SMethod, SMethodDesc},
                stype::SType,
                stype_companion::STypeCompanion,
            };
            let method_desc =
                SMethodDesc::property(SType::SHeader, "unknown", SType::SByte, MethodId(100));
            SMethod::new(STypeCompanion::Header, method_desc)
        };
        let expr = create_get_header_property_expr(unknown_property);
        assert!(try_eval_out_wo_ctx::<i8>(&expr).is_err());
    }
    #[test]
    fn test_eval_check_pow() {
        let mut ctx = force_any_val::<Context>();
        ctx.headers[0] = serde_json::from_str(
            r#"{
            "extensionId": "d51a477cc12b187d9bc7f464b22d00e3aa7c92463874e863bf3acf2f427bb48b",
            "difficulty": "1595361307131904",
            "votes": "000000",
            "timestamp": 1736177881102,
            "size": 220,
            "unparsedBytes": "",
            "stateRoot": "4dfafb43842680fd5870d8204a218f873479e1f5da1b34b059ca8da526abcc8719",
            "height": 1433531,
            "nBits": 117811961,
            "version": 3,
            "id": "3473e7b5aaf623e4260d5798253d26f3cdc912c12594b7e3a979e3db8ed883f6",
            "adProofsRoot": "73160faa9f0e47bf7da598d4e9d3de58e8a24b8564458ad8a4d926514f435dc1",
            "transactionsRoot": "c88d5f50ece85c2b918b5bd41d2bc06159e6db1b3aad95091d994c836a172950",
            "extensionHash": "d5a43bf63c1d8c7f10b15b6d2446abe565b93a4fd3f5ca785b00e6bda831644f",
            "powSolutions": {
              "pk": "0274e729bb6615cbda94d9d176a2f1525068f12b330e38bbbf387232797dfd891f",
              "w": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
              "n": "a6905b8c65f5864a",
              "d": 0
            },
            "adProofsId": "80a5ff0c6cd98440163bd27f2d7c775ea516af09024a98d9d83f16029bfbd034",
            "transactionsId": "c7315c49df258522d3e92ce2653d9f4d8a35309a7a7dd470ebf8db53dd3fb792",
            "parentId": "93172f3152a6a25dc89dc45ede1130c5eb86636a50bfb93a999556d16016ceb7"
          }"#,
        )
        .unwrap();
        // Add a mainnet block header with valid PoW to context. TODO: this can be simplified once Header serialization is added to sigma-rust (v6.0), right now we need to access CONTEXT.headers(0)
        let headers = PropertyCall::new(Expr::Context, HEADERS_PROPERTY.clone()).unwrap();
        let header = ByIndex::new(headers.into(), 0i32.into(), None).unwrap();
        let check_pow: Expr =
            MethodCall::new(header.into(), sheader::CHECK_POW_METHOD.clone(), vec![])
                .unwrap()
                .into();
        assert!(eval_out::<bool>(&check_pow, &ctx));
        // Mutate header to invalidate proof-of-work
        ctx.headers[0].timestamp -= 1;
        assert!(!eval_out::<bool>(&check_pow, &ctx));
    }
}
