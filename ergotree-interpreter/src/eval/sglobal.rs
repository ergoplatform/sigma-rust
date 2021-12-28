use crate::eval::EvalError;

use ergotree_ir::mir::value::{CollKind, NativeColl, Value};

use ergotree_ir::sigma_protocol::dlog_group::generator;

use super::EvalFn;

fn helper_xor(mut x: Vec<i8>, y: Vec<i8>) -> Vec<i8> {
    x.iter_mut().zip(y.iter()).for_each(|(x1, x2)| *x1 ^= *x2);
    x
}

pub(crate) static GROUP_GENERATOR_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.groupGenerator expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    Ok(Value::from(generator()))
};

pub(crate) static XOR_EVAL_FN: EvalFn = |_env, _ctx, obj, args| {
    if obj != Value::Global {
        return Err(EvalError::UnexpectedValue(format!(
            "sglobal.xor expected obj to be Value::Global, got {:?}",
            obj
        )));
    }
    let right_v = args
        .get(0)
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
            let xor = helper_xor(l_byte, r_byte);
            Ok(xor.into())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected Xor input to be byte array, got: {0:?}",
            (left_v, right_v)
        ))),
    }
};

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::sigma_protocol::dlog_group;
    use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::types::sglobal;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval_group_generator() {
        let expr: Expr = PropertyCall::new(Expr::Global, sglobal::GROUP_GENERATOR_METHOD.clone())
            .unwrap()
            .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<EcPoint>(&expr, ctx), dlog_group::generator());
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
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<Vec<i8>>(&expr, ctx), expected_xor);
    }
}
