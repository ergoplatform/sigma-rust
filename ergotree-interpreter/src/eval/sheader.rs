//! Evaluating predefined `Header` (or SHeader) type properties

use std::convert::TryInto;

use ergotree_ir::{bigint256::BigInt256, chain::header::Header, mir::constant::TryExtractInto};

use super::{EvalError, EvalFn};
use ergotree_ir::chain::block_id::BlockId;

pub(crate) static VERSION_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok((header.version as i8).into())
};

pub(crate) static ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    let BlockId(digest32) = header.id;
    Ok(Into::<Vec<i8>>::into(digest32).into())
};

pub(crate) static PARENT_ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    let BlockId(digest32) = header.parent_id;
    Ok(Into::<Vec<i8>>::into(digest32).into())
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
        mir::{
            coll_by_index::ByIndex, constant::TryExtractFromError, expr::Expr,
            property_call::PropertyCall,
        },
        sigma_protocol::dlog_group::EcPoint,
        types::{scontext, sheader, smethod::SMethod},
        util::AsVecU8,
    };
    use sigma_test_util::force_any_val;

    use crate::eval::{
        context::Context,
        tests::{eval_out, try_eval_out_wo_ctx},
        EvalError,
    };

    fn eval_header_pks(index: i32, ctx: Rc<Context>) -> [Box<EcPoint>; 2] {
        let get_headers_expr = create_get_header_by_index_expr(index);
        let miner_pk = eval_out::<EcPoint>(
            &create_header_property_call_expr(
                get_headers_expr.clone(),
                sheader::MINER_PK_PROPERTY.clone(),
            ),
            ctx.clone(),
        );
        let pow_onetime_pk = eval_out::<EcPoint>(
            &create_header_property_call_expr(
                get_headers_expr.clone(),
                sheader::POW_ONETIME_PK_PROPERTY.clone(),
            ),
            ctx.clone(),
        );
        [miner_pk, pow_onetime_pk].map(Box::new)
    }

    fn eval_header_roots(index: i32, ctx: Rc<Context>) -> [Digest32; 3] {
        let get_headers_expr = create_get_header_by_index_expr(index);
        vec![
            sheader::AD_PROOFS_ROOT_PROPERTY.clone(),
            sheader::TRANSACTIONS_ROOT_PROPERTY.clone(),
            sheader::EXTENSION_ROOT_PROPERTY.clone(),
        ]
        .into_iter()
        .map(|smethod| {
            eval_out::<Vec<i8>>(
                &create_header_property_call_expr(get_headers_expr.clone(), smethod),
                ctx.clone(),
            )
        })
        .map(digest_from_bytes_signed::<32>)
        .collect::<Vec<_>>()
        .try_into()
        .expect("internal error: smethods vector length is not equal to 3")
    }

    // Evaluates `Header.id` and `Header.parentId`
    fn eval_header_ids(index: i32, ctx: Rc<Context>) -> [BlockId; 2] {
        let get_headers_expr = create_get_header_by_index_expr(index);
        let id = eval_out::<Vec<i8>>(
            &create_header_property_call_expr(
                get_headers_expr.clone(),
                sheader::ID_PROPERTY.clone(),
            ),
            ctx.clone(),
        );
        let parent_id = eval_out::<Vec<i8>>(
            &create_header_property_call_expr(
                get_headers_expr,
                sheader::PARENT_ID_PROPERTY.clone(),
            ),
            ctx.clone(),
        );
        [id, parent_id].map(block_id_from_bytes_signed)
    }

    fn create_header_property_call_expr(headers_expr: Expr, method: SMethod) -> Expr {
        PropertyCall::new(headers_expr, method)
            .expect("internal error: invalid header property call")
            .into()
    }

    // An `Expr` for such code in ErgoScript `CONTEXT.headers(0)`
    fn create_get_header_by_index_expr(index: i32) -> Expr {
        let prop_call = PropertyCall::new(Expr::Context, scontext::HEADERS_PROPERTY.clone())
            .expect("internal error: invalid headers property call of Context")
            .into();
        ByIndex::new(prop_call, Expr::Const(index.into()), None)
            .expect("internal error: invalid types of ByIndex expression")
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
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::VERSION_PROPERTY.clone(),
        );
        let ctx = Rc::new(force_any_val::<Context>());
        let version = &ctx.headers[header_index as usize].version;
        assert_eq!(eval_out::<i8>(&expr, ctx.clone()), *version as i8);
    }

    #[test]
    fn test_eval_header_ids() {
        let header_index = 0;
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = {
            let h = ctx.headers[header_index as usize].clone();
            [h.id, h.parent_id]
        };
        let actual = eval_header_ids(header_index, ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_roots() {
        let header_index = 0;
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = {
            let h = ctx.headers[header_index as usize].clone();
            [h.ad_proofs_root, h.transaction_root, h.extension_root]
        };
        let actual = eval_header_roots(header_index, ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_state_root() {
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::STATE_ROOT_PROPERTY.clone(),
        );
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[header_index as usize].state_root.clone();
        let actual = digest_from_bytes_signed::<33>(eval_out::<Vec<i8>>(&expr, ctx.clone()));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_timestamp() {
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::TIMESTAMP_PROPERTY.clone(),
        );
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx
            .headers
            .get(0)
            .map(|h| h.timestamp as i64)
            .expect("internal error: empty headers array");
        let actual = eval_out::<i64>(&expr, ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_n_bits() {
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::N_BITS_PROPERTY.clone(),
        );
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx
            .headers
            .get(0)
            .map(|h| h.n_bits as i64)
            .expect("internal error: empty headers array");
        let actual = eval_out::<i64>(&expr, ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_height() {
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::HEIGHT_PROPERTY.clone(),
        );
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx
            .headers
            .get(0)
            .map(|h| h.height as i32)
            .expect("internal error: empty headers array");
        let actual = eval_out::<i32>(&expr, ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_pks() {
        let header_index = 0;
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx
            .headers
            .get(0)
            .map(|h| [h.miner_pk.clone(), h.pow_onetime_pk.clone()])
            .expect("internal error: empty headers array");
        let actual = eval_header_pks(header_index, ctx.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_pow_distance() {
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::POW_DISTANCE_PROPERTY.clone(),
        );
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[header_index as usize].pow_distance.clone();
        let actual = {
            let bi = eval_out::<BigInt256>(&expr, ctx.clone());
            bi.into()
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_pow_nonce() {
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::POW_NONCE_PROPERTY.clone(),
        );
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[header_index as usize].nonce.clone();
        let actual = eval_out::<Vec<i8>>(&expr, ctx.clone()).as_vec_u8();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eval_votes() {
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::VOTES_PROPERTY.clone(),
        );
        let ctx = Rc::new(force_any_val::<Context>());
        let expected = ctx.headers[header_index as usize].votes.clone();
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
        assert_eq!(
            try_eval_out_wo_ctx::<i8>(&expr),
            Err(EvalError::TryExtractFrom(TryExtractFromError(
                "expected Header, found Context".to_string()
            )))
        )
    }

    #[test]
    fn test_eval_failed_unknown_property() {
        let header_index = 0;
        let expr = create_header_property_call_expr(
            create_get_header_by_index_expr(header_index),
            sheader::UNKNOWN_PROPERTY.clone(),
        );
        assert_eq!(
            try_eval_out_wo_ctx::<i8>(&expr),
            Err(EvalError::NotFound(format!(
                "Eval fn: unknown method id in SHeader: {:?}",
                sheader::UNKNOWN_PROPERTY.method_id()
            )))
        )
    }
}
