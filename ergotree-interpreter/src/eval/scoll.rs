use crate::eval::EvalError;

use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;

use super::EvalFn;

pub static INDEX_OF_EVAL_FN: EvalFn = |_ctx, obj, args| {
    Ok(Value::Int({
        let normalized_input_vals: Vec<Value> = match obj {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected obj to be Value::Coll, got: {0:?}",
                obj
            ))),
        }?;
        let target_element = args
            .get(0)
            .cloned()
            .ok_or_else(|| EvalError::NotFound("indexOf: missing first arg".to_string()))?;
        let fallback_index = args
            .get(1)
            .cloned()
            .ok_or_else(|| EvalError::NotFound("indexOf: missing second arg".to_string()))?;
        let index_of = normalized_input_vals
            .into_iter()
            .position(|it| it == target_element)
            .unwrap_or(fallback_index.try_extract_into::<i32>()? as usize);
        index_of as i32
    }))
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    #![allow(unused_imports)]
    use ergotree_ir::mir::collection::Collection;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::types::sbox;
    use ergotree_ir::types::scoll;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::types::stype_param::STypeVar;
    use sigma_test_util::force_any_val;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out_wo_ctx;
    use std::rc::Rc;

    #[test]
    fn eval_index_of() {
        let coll_const: Constant = vec![1i64, 2i64].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::INDEX_OF_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::T, SType::SLong)].iter().cloned().collect()),
            vec![2i64.into(), 0i32.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<i32>(&expr);
        assert_eq!(res, 1);
    }

    #[test]
    fn eval_index_of_default() {
        let coll_const: Constant = vec![1i64, 2i64].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::INDEX_OF_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::T, SType::SLong)].iter().cloned().collect()),
            vec![3i64.into(), 0i32.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<i32>(&expr);
        assert_eq!(res, 0);
    }
}
