use crate::eval::EvalError;

use ergotree_ir::mir::value::Value;
use ergotree_ir::mir::xor::Xor;

use ergotree_ir::sigma_protocol::dlog_group::generator;


use super::EvalFn;

pub(crate) static GROUP_GENERATOR_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    if obj !=  Value::Global{
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.groupGenerator expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    Ok(Value::from(generator()))
};

pub(crate) static XOR_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    if obj != Value::Global{
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.xor expected obj to be Value::Global, got {:?}",
            obj
        )));
    }

    let left_v = obj.left.eval(env, ctx);
    let right_v = Xor.right.eval(env, ctx)?;

};



#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::types::sgroup_elem;

    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
    use sigma_test_util::force_any_val;
    use crate::eval::sglobal;

    #[test]
    fn eval_get_encoded() {
        let expr:Expr = PropertyCall::new(
            Expr::Global,
            sglobal::GROUP_GENERATOR_EVAL_FN.clone(),
        ).unwrap().into();
        let res:Vec<u8>=eval_out_wo_ctx(eval_out_wo_ctx::<&Expr>(&expr));
        assert!(!res.is_empty());
    }

}

