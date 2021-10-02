//! Evaluating predefined `Header` (or SHeader) type properties
#[allow(unused)] // TODO: remove attribute
use ergotree_ir::mir::value::Value;

use super::{EvalError, EvalFn};

pub(crate) static _VERSION_EVAL_FN: EvalFn = |_env, _ctx, _obj, _args| {
    // let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    return Err(EvalError::NotImplementedYet("just for tdd test"));
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    // use ergotree_ir::mir::{property_call::PropertyCall, expr::Expr};
    // use super::VERSION_EVAL_FN;
    //
    // #[test]
    // fn test_eval_version() {
    //     let expr: Expr =
    //         PropertyCall::new(Expr::Header, sheader::VERSION_PROPERTY.clone())
    //             .expect("internal error: invalid header version property call")
    //             .into();
    //     let ctx = Rc::new(force_any_val::<Context>());
    //     let header = ctx.headers.get(0).expect("internal error: empty headers array");
    //     assert_eq!(eval_out::<[Header; 10]>(&expr, ctx.clone()), ctx.headers);
    // }
}