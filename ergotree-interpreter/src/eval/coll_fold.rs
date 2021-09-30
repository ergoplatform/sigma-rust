use ergotree_ir::mir::coll_fold::Fold;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Fold {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let zero_v = self.zero.eval(env, ctx)?;
        let fold_op_v = self.fold_op.eval(env, ctx)?;
        let input_v_clone = input_v.clone();
        let mut fold_op_call = |arg: Value| match &fold_op_v {
            Value::Lambda(func_value) => {
                let func_arg = func_value
                    .args
                    .first()
                    .ok_or_else(|| EvalError::NotFound("empty argument for fold op".to_string()))?;
                let env1 = env.clone().extend(func_arg.idx, arg);
                func_value.body.eval(&env1, ctx)
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected fold_op to be Value::FuncValue got: {0:?}",
                input_v_clone
            ))),
        };
        match input_v {
            Value::Coll(coll) => match coll {
                CollKind::NativeColl(NativeColl::CollByte(coll_byte)) => {
                    coll_byte.iter().try_fold(zero_v, |acc, byte| {
                        let tup_arg = Value::Tup([acc, Value::Byte(*byte)].into());
                        fold_op_call(tup_arg)
                    })
                }
                CollKind::WrappedColl {
                    elem_tpe: _,
                    items: v,
                } => v.iter().try_fold(zero_v, |acc, item| {
                    let tup_arg = Value::Tup([acc, item.clone()].into());
                    fold_op_call(tup_arg)
                }),
            },
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected Fold input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::mir::bin_op::ArithOp;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::extract_amount::ExtractAmount;
    use ergotree_ir::mir::func_value::FuncArg;
    use ergotree_ir::mir::func_value::FuncValue;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::select_field::SelectField;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::types::scontext;
    use ergotree_ir::types::stuple::STuple;
    use ergotree_ir::types::stype::SType;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn eval_fold(ctx in any::<Context>()) {
            let data_inputs: Expr = PropertyCall::new(Expr::Context, scontext::DATA_INPUTS_PROPERTY.clone()).unwrap()
            .into();
            let tuple: Expr = ValUse {
                val_id: 1.into(),
                tpe: SType::STuple(STuple {
                    items: [SType::SLong, SType::SBox].into(),
                }),
            }
            .into();
            let fold_op_body: Expr = BinOp {
                kind: ArithOp::Plus.into(),
                left: Box::new(Expr::SelectField(
                    SelectField::new(tuple.clone(), 1.try_into().unwrap()).unwrap(),
                )),
                right: Box::new(Expr::ExtractAmount(
                    ExtractAmount::try_build(Expr::SelectField(
                        SelectField::new(tuple, 2.try_into().unwrap()).unwrap(),
                    ))
                    .unwrap(),
                )),
            }
            .into();
            let expr: Expr = Fold::new(
                data_inputs,
                Expr::Const(0i64.into()),
                FuncValue::new(
                    vec![FuncArg {
                        idx: 1.into(),
                        tpe: SType::STuple(STuple {
                        items: [SType::SLong, SType::SBox].into(),
                        }),
                    }],
                    fold_op_body,
                )
                .into(),
            )
            .unwrap()
            .into();
            let ctx = Rc::new(ctx);
            assert_eq!(
                eval_out::<i64>(&expr, ctx.clone()),
                ctx.data_inputs
                    .iter()
                    .fold(0i64, |acc, b| acc + b.value())
            );
        }

    }
}
