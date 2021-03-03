use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::if_op::If;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for If {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let condition_v = self.condition.eval(env, ctx)?;
        if condition_v.try_extract_into::<bool>()? {
            self.true_branch.eval(env, ctx)
        } else {
            self.false_branch.eval(env, ctx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::mir::bin_op::ArithOp;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::expr::Expr;

    #[test]
    fn eval() {
        let expr: Expr = If {
            condition: Expr::Const(true.into()).into(),
            true_branch: Expr::Const(1i64.into()).into(),
            false_branch: Expr::Const(2i64.into()).into(),
        }
        .into();
        let res = eval_out_wo_ctx::<i64>(&expr);
        assert_eq!(res, 1);
    }

    #[test]
    fn eval_laziness_true_branch() {
        let expr: Expr = If {
            condition: Expr::Const(true.into()).into(),
            true_branch: Expr::Const(1i64.into()).into(),
            false_branch: Box::new(
                BinOp {
                    kind: ArithOp::Divide.into(),
                    left: Box::new(Expr::Const(1i64.into())),
                    right: Box::new(Expr::Const(0i64.into())),
                }
                .into(),
            ),
        }
        .into();
        let res = eval_out_wo_ctx::<i64>(&expr);
        assert_eq!(res, 1);
    }

    #[test]
    fn eval_laziness_false_branch() {
        let expr: Expr = If {
            condition: Expr::Const(false.into()).into(),
            true_branch: Box::new(
                BinOp {
                    kind: ArithOp::Divide.into(),
                    left: Box::new(Expr::Const(1i64.into())),
                    right: Box::new(Expr::Const(0i64.into())),
                }
                .into(),
            ),
            false_branch: Expr::Const(1i64.into()).into(),
        }
        .into();
        let res = eval_out_wo_ctx::<i64>(&expr);
        assert_eq!(res, 1);
    }
}
