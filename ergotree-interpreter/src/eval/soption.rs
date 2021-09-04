use crate::eval::EvalError;
use crate::eval::Evaluable;

use ergotree_ir::mir::value::Value;

use super::EvalFn;

pub(crate) static MAP_EVAL_FN: EvalFn = |env, ctx, obj, args| {
    let input_v = obj;
    let lambda_v = args
        .get(0)
        .cloned()
        .ok_or_else(|| EvalError::NotFound("map: eval is missing first arg".to_string()))?;
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
            EvalError::NotFound("map: lambda has empty arguments list".to_string())
        })?;
        let env1 = env.clone().extend(func_arg.idx, arg);
        lambda.body.eval(&env1, ctx)
    };
    let normalized_input_val: Option<Value> = match input_v {
        Value::Opt(opt) => Ok(*opt),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected map input to be Value::Opt, got: {0:?}",
            input_v
        ))),
    }?;

    match normalized_input_val {
        Some(t) => Ok(Value::Opt(Box::new(lambda_call(t)?.into()))),
        _ => Ok(Value::Opt(Box::new(None))),
    }
};

pub(crate) static FILTER_EVAL_FN: EvalFn = |env, ctx, obj, args| {
    let input_v = obj;
    let lambda_v = args
        .get(0)
        .cloned()
        .ok_or_else(|| EvalError::NotFound("filter: eval is missing first arg".to_string()))?;
    let input_v_clone = input_v.clone();
    let lambda = match &lambda_v {
        Value::Lambda(l) => Ok(l),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected lambda to be Value::FuncValue got: {0:?}",
            input_v_clone
        ))),
    }?;
    let mut predicate_call = |arg: Value| {
        let func_arg = lambda.args.first().ok_or_else(|| {
            EvalError::NotFound("filter: lambda has empty arguments list".to_string())
        })?;
        let env1 = env.clone().extend(func_arg.idx, arg);
        lambda.body.eval(&env1, ctx)
    };
    let normalized_input_val: Option<Value> = match input_v {
        Value::Opt(opt) => Ok(*opt),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected filter input to be Value::Opt, got: {0:?}",
            input_v
        ))),
    }?;

    if let Some(v) = normalized_input_val {
        if let Value::Boolean(predicate_res) = predicate_call(v.clone())? {
            if predicate_res {
                return Ok(Value::Opt(Box::new(Some(v))));
            }
        }
    }
    Ok(Value::Opt(Box::new(None)))
};

#[allow(clippy::unwrap_used)]
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
    use ergotree_ir::types::soption;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::types::stype_param::STypeVar;

    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::mir::value::Value;

    #[test]
    fn eval_map_some() {
        let opt_const: Constant = Some(1i64).into();

        let body: Expr = BinOp {
            kind: RelationOp::Gt.into(),
            left: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SBox,
                }
                .into(),
            ),
            right: Box::new(Expr::Const(0i64.into())),
        }
        .into();

        let expr: Expr = MethodCall::new(
            opt_const.into(),
            soption::MAP_METHOD.clone().with_concrete_types(
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

        let res = eval_out_wo_ctx::<Value>(&expr);
        assert_eq!(
            res,
            Value::Opt(Box::new(Option::Some(Value::Boolean(true))))
        );
    }

    #[test]
    fn eval_map_none() {
        let typed_none: Option<i64> = None;
        let opt_const: Constant = typed_none.into();

        let body: Expr = BinOp {
            kind: RelationOp::Gt.into(),
            left: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SBox,
                }
                .into(),
            ),
            right: Box::new(Expr::Const(0i64.into())),
        }
        .into();

        let expr: Expr = MethodCall::new(
            opt_const.into(),
            soption::MAP_METHOD.clone().with_concrete_types(
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

        let res = eval_out_wo_ctx::<Value>(&expr);
        assert_eq!(res, Value::Opt(Box::new(None)));
    }

    #[test]
    fn eval_filter_some_true() {
        let opt_const: Constant = Some(1i64).into();

        let body: Expr = BinOp {
            kind: RelationOp::Gt.into(),
            left: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SBox,
                }
                .into(),
            ),
            right: Box::new(Expr::Const(0i64.into())),
        }
        .into();

        let expr: Expr = MethodCall::new(
            opt_const.into(),
            soption::FILTER_METHOD.clone().with_concrete_types(
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

        let res = eval_out_wo_ctx::<Value>(&expr);
        assert_eq!(res, Value::Opt(Box::new(Option::Some(Value::Long(1)))));
    }

    #[test]
    fn eval_filter_some_false() {
        let opt_const: Constant = Some(1i64).into();

        let body: Expr = BinOp {
            kind: RelationOp::Gt.into(),
            left: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SBox,
                }
                .into(),
            ),
            right: Box::new(Expr::Const(10i64.into())),
        }
        .into();

        let expr: Expr = MethodCall::new(
            opt_const.into(),
            soption::FILTER_METHOD.clone().with_concrete_types(
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

        let res = eval_out_wo_ctx::<Value>(&expr);
        assert_eq!(res, Value::Opt(Box::new(Option::None)));
    }

    #[test]
    fn eval_filter_none() {
        let typed_none: Option<i64> = None;
        let opt_const: Constant = typed_none.into();

        let body: Expr = BinOp {
            kind: RelationOp::Gt.into(),
            left: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SBox,
                }
                .into(),
            ),
            right: Box::new(Expr::Const(0i64.into())),
        }
        .into();

        let expr: Expr = MethodCall::new(
            opt_const.into(),
            soption::FILTER_METHOD.clone().with_concrete_types(
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

        let res = eval_out_wo_ctx::<Value>(&expr);
        assert_eq!(res, Value::Opt(Box::new(Option::None)));
    }
}
