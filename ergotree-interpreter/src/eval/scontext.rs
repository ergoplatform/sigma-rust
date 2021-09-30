use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

use super::EvalError;
use super::EvalFn;

pub(crate) static DATA_INPUTS_EVAL_FN: EvalFn = |_env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.dataInputs: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    Ok(Value::Coll(CollKind::WrappedColl {
        items: ctx
            .ctx
            .data_inputs
            .clone()
            .into_iter()
            .map(Value::CBox)
            .collect(),
        elem_tpe: SType::SBox,
    }))
};

pub(crate) static SELF_BOX_INDEX_EVAL_FN: EvalFn = |_env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.selfBoxIndex: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    let box_index = ctx
        .ctx
        .inputs
        .clone()
        .iter()
        .position(|it| it == &ctx.ctx.self_box)
        .ok_or_else(|| EvalError::NotFound("Context.selfBoxIndex: box not found".to_string()))?;
    Ok(Value::Int(box_index as i32))
};

pub(crate) static HEADERS_EVAL_FN: EvalFn = |_env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.headers: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    Ok(Value::Coll(CollKind::WrappedColl {
        items: ctx.ctx.headers.clone().map(Value::Header).to_vec(),
        elem_tpe: SType::SHeader,
    }))
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::eval::context::ir_ergo_box_dummy::IrErgoBoxDummy;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::chain::header::Header;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::types::scontext;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    fn make_ctx_inputs_includes_self_box() -> Context {
        let ctx = force_any_val::<Context>();
        let self_box = force_any_val::<IrErgoBoxDummy>();
        let inputs = vec![force_any_val::<IrErgoBoxDummy>().id, self_box.id.clone()];
        Context {
            height: 0u32,
            self_box: self_box.id,
            inputs,
            ..ctx
        }
    }

    #[test]
    fn eval_self_box_index() {
        let expr: Expr =
            PropertyCall::new(Expr::Context, scontext::SELF_BOX_INDEX_PROPERTY.clone())
                .unwrap()
                .into();
        let rc = Rc::new(make_ctx_inputs_includes_self_box());
        assert_eq!(eval_out::<i32>(&expr, rc), 1);
    }

    #[test]
    fn eval_headers() {
        let expr: Expr = PropertyCall::new(Expr::Context, scontext::HEADERS_PROPERTY.clone())
            .expect("internal error: `headers` method has parameters length != 1")
            .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<[Header; 10]>(&expr, ctx.clone()), ctx.headers);
    }
}
