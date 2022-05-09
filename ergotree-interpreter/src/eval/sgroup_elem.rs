use crate::eval::EvalError;

use ergo_chain_types::EcPoint;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaSerializable;

use super::EvalFn;

pub(crate) static GET_ENCODED_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let encoded: Vec<u8> = match obj {
        Value::GroupElement(ec_point) => Ok(ec_point.sigma_serialize_bytes()?),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected obj to be Value::GroupElement, got: {0:?}",
            obj
        ))),
    }?;

    Ok(Value::from(encoded))
};

pub(crate) static NEGATE_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let negated: EcPoint = match obj {
        Value::GroupElement(ec_point) => Ok(-(*ec_point)),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected obj to be Value::GroupElement, got: {0:?}",
            obj
        ))),
    }?;
    Ok(Value::GroupElement(Box::new(negated)))
};

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::types::sgroup_elem;

    use crate::eval::tests::eval_out_wo_ctx;
    use ergo_chain_types::EcPoint;
    use ergotree_ir::serialization::SigmaSerializable;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval_get_encoded() {
        let input = force_any_val::<EcPoint>();
        let expr: Expr = MethodCall::new(
            input.clone().into(),
            sgroup_elem::GET_ENCODED_METHOD.clone(),
            vec![],
        )
        .unwrap()
        .into();

        let res: Vec<u8> = eval_out_wo_ctx::<Vec<u8>>(&expr);
        let roundtrip_res: EcPoint = SigmaSerializable::sigma_parse_bytes(&res).unwrap();

        assert!(!res.is_empty());
        assert_eq!(input, roundtrip_res)
    }

    #[test]
    fn eval_negate() {
        let input = force_any_val::<EcPoint>();
        let expr: Expr = MethodCall::new(
            input.clone().into(),
            sgroup_elem::NEGATE_METHOD.clone(),
            vec![],
        )
        .unwrap()
        .into();
        assert_eq!(-input, eval_out_wo_ctx::<EcPoint>(&expr))
    }
}
