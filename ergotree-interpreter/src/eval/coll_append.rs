use ergotree_ir::mir::coll_append::Append;
// use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;
use ergotree_ir::mir::value::CollKind;


use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

fn _helper_concat<T>(mut e1 : Vec<T>, e2 : Vec<T>) -> Vec<T> {
    e1.extend(e2);
    e1
}

impl Evaluable for Append {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        // let vec = vec![];
        // Ok(Value::from(vec![1_u8]))
        let input_val : Value = self.input.eval(env, ctx)?;
        let output_val : Value = self.output.eval(env, ctx)?;
        let input_vecval : Vec<Value> = match input_val {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Append: expected input to be Value::Coll, got: {0:?}",
                input_val
            ))),
        }?;
        let output_vecval : Vec<Value> = match output_val {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Append: expected output to be Value::Coll, got: {0:?}",
                output_val
            ))),
        }?;
        let concat_vecval : Vec<Value> = _helper_concat(input_vecval, output_vecval);
        Ok(Value::Coll(CollKind::from_vec(self.tpe(), concat_vecval)?))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    // use ergotree_ir::ir_ergo_box::IrBoxId;
    use ergotree_ir::mir::expr::Expr;
    // use ergotree_ir::mir::global_vars::GlobalVars;
    // use sigma_test_util::force_any_val;

    use super::*;
    // use crate::eval::context::Context;
    // use crate::eval::tests::eval_out;
    use crate::eval::tests::eval_out_wo_ctx;
    // use std::rc::Rc;

    #[test]
    fn basic_eval() {

        let in_expr : Expr = Expr::from(vec![1,2,3,4]);
        let out_expr : Expr = Expr::from(vec![5,6,7,8]);
        let construct_append : Expr = Append::new(in_expr, out_expr).unwrap().into();
        let append_eval = eval_out_wo_ctx::<Vec<i32>>(&construct_append);
        // let append_expr : Value = .eval(ctx, env).unwrap();
        let concat_expr = Expr::from(vec![1,2,3,4,5,6,7,8]);
        let concat_eval = eval_out_wo_ctx::<Vec<i32>>(&concat_expr);

        assert_eq!(append_eval, concat_eval);
        // let expr: Expr = Append::new(GlobalVars::Inputs.into(), GlobalVars::Outputs.into())
        //     .unwrap()
        //     .into();
        // let ctx = Rc::new(force_any_val::<Context>());
        // assert_eq!(
        //     eval_out::<IrBoxId>(&expr, ctx.clone()),
        //     ctx.outputs.get(0).unwrap().clone()
        // );
    }

   
}


        // #[test]
        // fn eval_box_value(ctx in any::<Context>()) {
        //     let data_inputs: Expr = PropertyCall::new(Expr::Context, scontext::DATA_INPUTS_PROPERTY.clone()).unwrap()
        //     .into();
        //     let val_use: Expr = ValUse {
        //         val_id: 1.into(),
        //         tpe: SType::SBox,
        //     }
        //     .into();
        //     let mapper_body: Expr = BinOp {
        //         kind: ArithOp::Plus.into(),
        //         left: Box::new(Expr::Const(1i64.into())),
        //         right: Box::new(Expr::ExtractAmount(
        //                 ExtractAmount::try_build(val_use)
        //             .unwrap(),
        //         )),
        //     }
        //     .into();
        //     let expr: Expr = Map::new(
        //         data_inputs,
        //         FuncValue::new(
        //             vec![FuncArg {
        //                 idx: 1.into(),
        //                 tpe: SType::SBox,
        //             }],
        //             mapper_body,
        //         )
        //         .into(),
        //     )
        //     .unwrap()
        //     .into();
        //     let ctx = Rc::new(ctx);
        //     assert_eq!(
        //         eval_out::<Vec<i64>>(&expr, ctx.clone()),
        //         ctx.data_inputs
        //             .iter()
        //             .map(| b| b.get_box(&ctx.box_arena).unwrap().value() + 1).collect::<Vec<i64>>()
        //     );
        // }
