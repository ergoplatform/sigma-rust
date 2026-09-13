use ergotree_ir::mir::coll_fold::Fold;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Fold {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let zero_v = self.zero.eval(env, ctx)?;
        let fold_op_v = self.fold_op.eval(env, ctx)?;
        let input_v_clone = input_v.clone();
        let mut fold_op_call = |arg: Value<'ctx>| match &fold_op_v {
            Value::Lambda(func_value) => crate::eval::eval_lambda_1arg(
                func_value,
                arg,
                env,
                ctx,
                "empty argument for fold op",
            ),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected fold_op to be Value::FuncValue got: {0:?}",
                input_v_clone
            ))),
        };
        let n_items = match &input_v {
            Value::Coll(coll) => coll.len() as u32,
            _ => 0,
        };
        ctx.add_per_item_jit_cost(3, 1, 10, n_items)?;
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
#[cfg(feature = "arbitrary")]
mod tests {
    use core::convert::TryInto;

    use crate::eval::test_util::eval_out;
    use ergotree_ir::chain::context::Context;
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
                    SelectField::new(tuple.clone(), 1.try_into().unwrap()).unwrap().into(),
                )),
                right: Box::new(Expr::ExtractAmount(
                    ExtractAmount::try_build(Expr::SelectField(
                        SelectField::new(tuple, 2.try_into().unwrap()).unwrap().into(),
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
            assert_eq!(
                eval_out::<i64>(&expr, &ctx),
                ctx.data_inputs.clone()
                    .map_or(0i64, |d| d.iter().fold(0i64, |acc, b| acc + b.value.as_i64()))
            );
        }

    }
}
