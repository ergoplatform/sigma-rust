use crate::eval::EvalError;
use crate::eval::Evaluable;

use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;

use super::EvalFn;

pub(crate) static INDEX_OF_EVAL_FN: EvalFn = |_env, _ctx, obj, args| {
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

pub(crate) static FLATMAP_EVAL_FN: EvalFn = |env, ctx, obj, args| {
    let input_v = obj;
    let lambda_v = args
        .get(0)
        .cloned()
        .ok_or_else(|| EvalError::NotFound("flatmap: eval is missing first arg".to_string()))?;
    let input_v_clone = input_v.clone();
    let lambda = match &lambda_v {
        Value::Lambda(l) => Ok(l),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected lambda to be Value::FuncValue got: {0:?}",
            input_v_clone
        ))),
    }?;
    let mut lambda_call = |arg: Value| {
        let func_arg = lambda.args.first().ok_or_else(|| {
            EvalError::NotFound("flatmap: lambda has empty arguments list".to_string())
        })?;
        let env1 = env.clone().extend(func_arg.idx, arg);
        lambda.body.eval(&env1, ctx)
    };
    let mapper_input_tpe = lambda
        .args
        .first()
        .cloned()
        .map(|arg| arg.tpe)
        .ok_or_else(|| {
            EvalError::NotFound(
                "flatmap: lambda args are empty (does not have arguments)".to_string(),
            )
        })?;
    let normalized_input_vals: Vec<Value> = match input_v {
        Value::Coll(coll) => {
            if *coll.elem_tpe() != mapper_input_tpe {
                return Err(EvalError::UnexpectedValue(format!(
                    "expected Flatmap input element type to be {0:?}, got: {1:?}",
                    mapper_input_tpe,
                    coll.elem_tpe()
                )));
            };
            Ok(coll.as_vec())
        }
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected Flatmap input to be Value::Coll, got: {0:?}",
            input_v
        ))),
    }?;
    normalized_input_vals
        .iter()
        .map(|item| lambda_call(item.clone()))
        .collect::<Result<Vec<Value>, EvalError>>()
        .map(|values| {
            CollKind::from_vec(lambda.body.tpe(), values).map_err(EvalError::TryExtractFrom)
        })
        .and_then(|v| v) // flatten <Result<Result<Value, _>, _>
        .map(Value::Coll)
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::bin_op::RelationOp;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::func_value::FuncArg;
    use ergotree_ir::mir::func_value::FuncValue;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::types::scoll;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::types::stype_param::STypeVar;

    use crate::eval::tests::eval_out_wo_ctx;

    #[test]
    fn eval_index_of() {
        let coll_const: Constant = vec![1i64, 2i64].into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::INDEX_OF_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
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
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![3i64.into(), 0i32.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<i32>(&expr);
        assert_eq!(res, 0);
    }

    #[test]
    fn eval_flatmap() {
        let coll_const: Constant = vec![1i64, 2i64].into();
        let body: Expr = BinOp {
            kind: RelationOp::Ge.into(),
            left: Box::new(Expr::Const(1i64.into())),
            right: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SBox,
                }
                .into(),
            ),
        }
        .into();
        let expr: Expr = MethodCall::new(
            coll_const.into(),
            scoll::FLATMAP_METHOD.clone().with_concrete_types(
                &[
                    (STypeVar::iv(), SType::SLong),
                    (STypeVar::ov(), SType::SBoolean),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            vec![FuncValue::new(
                vec![FuncArg {
                    idx: 1.into(),
                    tpe: SType::SLong,
                }],
                body,
            )
            .into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Vec<bool>>(&expr);
        assert_eq!(res, vec![true, false]);
    }
}
