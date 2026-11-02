use crate::eval::EvalError;
use crate::eval::LambdaInvoker;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::smethod::SMethod;

use super::env::Env;
use super::Context;

pub fn map_eval<'ctx>(
    _mc: &SMethod,
    _env: &mut Env<'ctx>,
    ctx: &Context<'ctx>,
    obj: Value<'ctx>,
    args: Vec<Value<'ctx>>,
) -> Result<Value<'ctx>, EvalError> {
    let input_v = obj;
    let lambda_v = args
        .first()
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
    lambda
        .args
        .first()
        .ok_or_else(|| EvalError::NotFound("map: lambda has empty arguments list".to_string()))?;
    // The body evaluates in the lambda's CAPTURED environment (JVM closure
    // semantics) — not in the caller's env.
    let mut invoker = LambdaInvoker::new(lambda);
    let mut lambda_call = |arg: Value<'ctx>| invoker.invoke(ctx, vec![arg]);
    let normalized_input_val: Option<Value> = match input_v {
        Value::Opt(opt) => Ok(opt.as_deref().cloned()),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected map input to be Value::Opt, got: {0:?}",
            input_v
        ))),
    }?;

    match normalized_input_val {
        Some(t) => Ok(Value::Opt(Box::new(lambda_call(t)?).into())),
        _ => Ok(Value::Opt(None)),
    }
}

pub fn filter_eval<'ctx>(
    _mc: &SMethod,
    _env: &mut Env<'ctx>,
    ctx: &Context<'ctx>,
    obj: Value<'ctx>,
    args: Vec<Value<'ctx>>,
) -> Result<Value<'ctx>, EvalError> {
    let input_v = obj;
    let lambda_v = args
        .first()
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
    lambda.args.first().ok_or_else(|| {
        EvalError::NotFound("filter: lambda has empty arguments list".to_string())
    })?;
    // The body evaluates in the lambda's CAPTURED environment (JVM closure
    // semantics) — not in the caller's env.
    let mut invoker = LambdaInvoker::new(lambda);
    let mut predicate_call = |arg: Value<'ctx>| invoker.invoke(ctx, vec![arg]);
    let normalized_input_val: Option<Value> = match input_v {
        Value::Opt(opt) => Ok(opt.as_deref().cloned()),
        _ => Err(EvalError::UnexpectedValue(format!(
            "expected filter input to be Value::Opt, got: {0:?}",
            input_v
        ))),
    }?;

    match normalized_input_val {
        Some(val) => match predicate_call(val.clone())? {
            Value::Boolean(predicate_res) => match predicate_res {
                true => Ok(Value::Opt(Some(Box::new(val)))),
                false => Ok(Value::Opt(None)),
            },
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected filter predicate result to be boolean, got: {0:?}",
                lambda.body.tpe()
            ))),
        },
        None => Ok(Value::Opt(None)),
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use alloc::boxed::Box;
    use ergotree_ir::mir::bin_op::RelationOp;
    use ergotree_ir::mir::bin_op::{ArithOp, BinOp};
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::func_value::FuncArg;
    use ergotree_ir::mir::func_value::FuncValue;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::types::soption;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::types::stype_param::STypeVar;

    use crate::eval::test_util::eval_out_wo_ctx;
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
            Value::Opt(Option::Some(Box::new(Value::Boolean(true))))
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
        assert_eq!(res, Value::Opt(None));
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
            soption::FILTER_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::iv(), SType::SLong)].iter().cloned().collect()),
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
        assert_eq!(res, Value::Opt(Option::Some(Box::new(Value::Long(1)))));
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
            soption::FILTER_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::iv(), SType::SLong)].iter().cloned().collect()),
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
        assert_eq!(res, Value::Opt(Option::None));
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
            soption::FILTER_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::iv(), SType::SLong)].iter().cloned().collect()),
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
        assert_eq!(res, Value::Opt(Option::None));
    }

    #[test]
    fn eval_filter_predicate_invalid_tpe() {
        let opt_const: Constant = Some(1i64).into();

        let body: Expr = BinOp {
            kind: ArithOp::Plus.into(),
            left: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SBox,
                }
                .into(),
            ),
            right: Box::new(Expr::Const(2i64.into())),
        }
        .into();
        body.tpe();

        assert!(MethodCall::new(
            opt_const.into(),
            soption::FILTER_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::iv(), SType::SLong)].iter().cloned().collect()),
            vec![FuncValue::new(
                vec![FuncArg {
                    idx: 1.into(),
                    tpe: SType::SLong,
                }],
                body,
            )
            .into()],
        )
        .is_err());
    }
}
