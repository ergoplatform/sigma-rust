//! Evaluating predefined `Header` (or SHeader) type properties

use ergotree_ir::chain::header::Header;
use ergotree_ir::mir::{constant::TryExtractInto, value::Value};

use super::EvalFn;

pub(crate) static VERSION_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let header = obj.try_extract_into::<Header>()?;
    // TODO: [sab] this is actually not accurate, fix it!
    Ok(Value::Byte(header.version as i8))
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::rc::Rc;
    
    use ergotree_ir::mir::{coll_by_index::ByIndex, expr::Expr, property_call::PropertyCall};
    use ergotree_ir::types::{scontext, sheader};
    use sigma_test_util::force_any_val;

    use crate::eval::{context::Context, tests::eval_out};

    // An `Expr` for such code in ErgoScript `CONTEXT.headers(0)`
    fn get_header_by_index_expr(index: i32) -> Expr {
        let prop_call = PropertyCall::new(Expr::Context, scontext::HEADERS_PROPERTY.clone())
            .expect("internal error: invalid headers property call of Context")
            .into();
        ByIndex::new(prop_call, Expr::Const(index.into()), None)
            .expect("internal error: invalid types of ByIndex expression")
            .into()
    }

    #[test]
    fn test_eval_version() {
        let header_index = 0;
        let expr: Expr = PropertyCall::new(
            get_header_by_index_expr(header_index),
            sheader::VERSION_PROPERTY.clone(),
        )
        .expect("internal error: invalid header version property call")
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        let version = &ctx.headers[header_index as usize].version;
        // TODO: [sab] this is actually not accurate, fix it!
        assert_eq!(eval_out::<i8>(&expr, ctx.clone()), *version as i8);
    }
}
