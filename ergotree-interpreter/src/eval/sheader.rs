//! Evaluating predefined `Header` (or SHeader) type properties

use ergotree_ir::chain::header::Header;
use ergotree_ir::mir::{constant::TryExtractInto, value::Value};

use super::EvalFn;

pub(crate) static VERSION_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(Value::Byte(header.version as i8))
};

pub(crate) static ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(header.id.into_bytes_signed().into())
};

pub(crate) static PARENT_ID_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    Ok(header.parent_id.into_bytes_signed().into())
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

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::convert::TryInto;
    use std::rc::Rc;

    use ergotree_ir::chain::{
        block_id::BlockId,
        digest32::{Digest, Digest32},
    };
    use ergotree_ir::mir::{coll_by_index::ByIndex, expr::Expr, property_call::PropertyCall};
    use ergotree_ir::types::{scontext, sheader, smethod::SMethod};
    use ergotree_ir::util::AsVecU8;
    use sigma_test_util::force_any_val;

    use crate::eval::{context::Context, tests::eval_out};

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
}
