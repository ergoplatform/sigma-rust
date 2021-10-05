//! Evaluating predefined `Header` (or SHeader) type properties

use std::convert::TryInto;

use ergotree_ir::{bigint256::BigInt256, chain::header::Header, mir::constant::TryExtractInto};

use super::{EvalError, EvalFn};

pub(crate) static VERSION_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.version as i8).into())
};

pub(crate) static ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.id).into())
};

pub(crate) static PARENT_ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.parent_id).into())
};

pub(crate) static AD_PROOFS_ROOT_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.ad_proofs_root).into())
};

pub(crate) static STATE_ROOT_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.state_root).into())
};

pub(crate) static TRANSACTION_ROOT_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.transaction_root).into())
};

pub(crate) static EXTENSION_ROOT_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<i8>>::into(header.extension_root).into())
};

pub(crate) static TIMESTAMP_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.timestamp as i64).into())
};

pub(crate) static N_BITS_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.n_bits as i64).into())
};

pub(crate) static HEIGHT_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.height as i32).into())
};

pub(crate) static MINER_PK_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(header.miner_pk.into())
};

pub(crate) static POW_ONETIME_PK_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(header.pow_onetime_pk.into())
};

pub(crate) static POW_NONCE_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(header.nonce.into())
};

pub(crate) static POW_DISTANCE_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    let pow_distance: BigInt256 = header.pow_distance.try_into().map_err(EvalError::Misc)?;
    Ok(pow_distance.into())
};

pub(crate) static VOTES_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(Into::<Vec<u8>>::into(header.votes).into())
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::expect_used)]
mod tests {
    use std::convert::{TryFrom, TryInto};
    use std::rc::Rc;

    use ergotree_ir::{
        bigint256::BigInt256,
        chain::{
            block_id::BlockId,
            digest32::{Digest, Digest32},
            votes::Votes,
        },
        mir::{coll_by_index::ByIndex, expr::Expr, property_call::PropertyCall},
        sigma_protocol::dlog_group::EcPoint,
        types::{scontext, sheader, smethod::SMethod},
        util::AsVecU8,
    };
    use sigma_test_util::force_any_val;

    use crate::eval::{
        context::Context,
        tests::{eval_out, try_eval_out_wo_ctx},
    };

    // Index in Context.headers array
    const HEADER_INDEX: usize = 0;

    // Evaluates `Header.minerPk`, `Header.powOnetimePk`
    fn eval_header_pks(ctx: Rc<Context>) -> [Box<EcPoint>; 2] {
        let miner_pk = eval_out::<EcPoint>(
            &create_get_header_property_expr(sheader::MINER_PK_PROPERTY.clone()),
            ctx.clone(),
        );
        let pow_onetime_pk = eval_out::<EcPoint>(
            &create_get_header_property_expr(sheader::POW_ONETIME_PK_PROPERTY.clone()),
            ctx.clone(),
        );
        [miner_pk, pow_onetime_pk].map(Box::new)
    }

    // Evaluates `Header.AdProofsRoot`, `Header.transactionRoot`, `Header.extensionRoot`
    fn eval_header_roots(ctx: Rc<Context>) -> [Digest32; 3] {
        vec![
            sheader::AD_PROOFS_ROOT_PROPERTY.clone(),
            sheader::TRANSACTIONS_ROOT_PROPERTY.clone(),
            sheader::EXTENSION_ROOT_PROPERTY.clone(),
        ]
        .into_iter()
        .map(|smethod| eval_out::<Vec<i8>>(&create_get_header_property_expr(smethod), ctx.clone()))
        .map(digest_from_bytes_signed::<32>)
        .collect::<Vec<_>>()
        .try_into()
        .expect("internal error: smethods vector length is not equal to 3")
    }

    // Evaluates `Header.id` and `Header.parentId`
    fn eval_header_ids(ctx: Rc<Context>) -> [BlockId; 2] {
        let id = eval_out::<Vec<i8>>(
            &create_get_header_property_expr(sheader::ID_PROPERTY.clone()),
            ctx.clone(),
        );
        let parent_id = eval_out::<Vec<i8>>(
            &create_get_header_property_expr(sheader::PARENT_ID_PROPERTY.clone()),
            ctx.clone(),
        );
        [id, parent_id].map(block_id_from_bytes_signed)
    }

    fn create_get_header_property_expr(method: SMethod) -> Expr {
        let get_headers_expr = create_get_header_by_index_expr();
        create_header_property_call_expr(get_headers_expr.clone(), method)
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

    fn digest_from_bytes_signed<const N: usize>(bytes: Vec<i8>) -> Digest<N> {
        let arr = arr_from_bytes_signed::<N>(bytes);
        arr.into()
    }

    fn block_id_from_bytes_signed(bytes: Vec<i8>) -> BlockId {
        let arr32 = digest_from_bytes_signed::<32>(bytes);
        BlockId(arr32)
    }

    fn arr_from_bytes_signed<const N: usize>(bytes: Vec<i8>) -> [u8; N] {
        bytes.as_vec_u8().try_into().expect(&format!(
            "internal error: bytes buffer length is not equal to {}",
            N
        ))
    }

    #[test]
    fn test_eval_header_version() {
        let expr = create_get_header_property_expr(sheader::VERSION_PROPERTY.clone());
        let ctx = Rc::new(force_any_val::<Context>());
        let version = ctx.headers[HEADER_INDEX].version;
        assert_eq!(version as i8, eval_out::<i8>(&expr, ctx.clone()));
    }

    #[test]
    fn test_eval_header_ids() {
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx
            .headers
            .get(HEADER_INDEX)
            .map(|h| [h.id.clone(), h.parent_id.clone()])
            .expect("internal error: empty headers array");
        let actual = eval_header_ids(ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_roots() {
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx
            .headers
            .get(HEADER_INDEX)
            .map(|h| {
                [
                    h.ad_proofs_root.clone(),
                    h.transaction_root.clone(),
                    h.extension_root.clone(),
                ]
            })
            .expect("internal error: empty headers array");
        let actual = eval_header_roots(ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_state_root() {
        let expr = create_get_header_property_expr(sheader::STATE_ROOT_PROPERTY.clone());
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[HEADER_INDEX].state_root.clone();
        let actual = digest_from_bytes_signed::<33>(eval_out::<Vec<i8>>(&expr, ctx.clone()));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_timestamp() {
        let expr = create_get_header_property_expr(sheader::TIMESTAMP_PROPERTY.clone());
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[HEADER_INDEX].timestamp;
        let actual = eval_out::<i64>(&expr, ctx.clone());
        assert_eq!(expected as i64, actual);
    }

    #[test]
    fn test_eval_n_bits() {
        let expr = create_get_header_property_expr(sheader::N_BITS_PROPERTY.clone());
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[HEADER_INDEX].n_bits;
        let actual = eval_out::<i64>(&expr, ctx.clone());
        assert_eq!(expected as i64, actual);
    }

    #[test]
    fn test_eval_height() {
        let expr = create_get_header_property_expr(sheader::HEIGHT_PROPERTY.clone());
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[HEADER_INDEX].height;
        let actual = eval_out::<i32>(&expr, ctx.clone());
        assert_eq!(expected as i32, actual);
    }

    #[test]
    fn test_eval_pks() {
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx
            .headers
            .get(HEADER_INDEX)
            .map(|h| [h.miner_pk.clone(), h.pow_onetime_pk.clone()])
            .expect("internal error: empty headers array");
        let actual = eval_header_pks(ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_pow_distance() {
        let expr = create_get_header_property_expr(sheader::POW_DISTANCE_PROPERTY.clone());
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[HEADER_INDEX].pow_distance.clone();
        let actual = {
            let bi = eval_out::<BigInt256>(&expr, ctx.clone());
            bi.into()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_pow_nonce() {
        let expr = create_get_header_property_expr(sheader::POW_NONCE_PROPERTY.clone());
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[HEADER_INDEX].nonce.clone();
        let actual = eval_out::<Vec<i8>>(&expr, ctx.clone()).as_vec_u8();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_votes() {
        let expr = create_get_header_property_expr(sheader::VOTES_PROPERTY.clone());
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[HEADER_INDEX].votes.clone();
        let actual = {
            let votes_bytes = eval_out::<Vec<i8>>(&expr, ctx.clone()).as_vec_u8();
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
}
