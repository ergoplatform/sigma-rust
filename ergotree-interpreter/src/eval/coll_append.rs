use ergotree_ir::mir::coll_append::Append;
// use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

fn concat<T>(mut e1: Vec<T>, e2: Vec<T>) -> Vec<T> {
    e1.extend(e2);
    e1
}

fn extract_vecval(inp: Value) -> Result<Vec<Value>, EvalError> {
    match inp {
        Value::Coll(coll) => Ok(coll.as_vec()),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Append: expected Value to be Value::Coll, got: {0:?}",
            inp
        ))),
    }
}

impl Evaluable for Append {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_vecval: Vec<Value> = extract_vecval(self.input.eval(env, ctx)?)?;
        let col_2_vecval: Vec<Value> = extract_vecval(self.col_2.eval(env, ctx)?)?;
        let concat_vecval: Vec<Value> = concat(input_vecval, col_2_vecval);
        Ok(Value::Coll(CollKind::from_vec(self.tpe(), concat_vecval)?))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::mir::expr::Expr;

    // Comment here for an elegant solution - generic single_test
    // Fails due to Expr::from needing complicated type bounds
    // Fails both with T and Vec<T>
    // fn test_single_append<T : PartialEq + Debug + LiftIntoSType> (inp : T, inp2 : T, expected_output : T) {
    //     let in_expr = Expr::from(inp);
    //     let col2_expr = Expr::from(inp2);
    //     let append_expr = Expr::from(Append::new(in_expr, col2_expr).unwrap());
    //     let append_eval : Vec<T> = eval_out_wo_ctx(append_expr);
    //     assert_eq!(append_eval, expected_output);
    // }

    #[test]
    fn test_append_basic_numbers() {
        for (test_inp, test_col2, expected_output) in [
            (
                vec![1, 2, 3, 4],
                vec![5, 6, 7, 8],
                vec![1, 2, 3, 4, 5, 6, 7, 8],
            ),
            (vec![], vec![], vec![]),
            (vec![], vec![1], vec![1]),
            (vec![], vec![2, 2], vec![2, 2]),
            (vec![1], vec![], vec![1]),
            (vec![2, 2], vec![], vec![2, 2]),
            (vec![1], vec![2, 2], vec![1, 2, 2]),
        ]
        .iter()
        {
            let in_expr = Expr::from(test_inp.clone());
            let col2_expr = Expr::from(test_col2.clone());
            let append_expr = Expr::from(Append::new(in_expr, col2_expr).unwrap());
            let append_eval: Vec<i32> = eval_out_wo_ctx(&append_expr);
            assert_eq!(append_eval, *expected_output);
        }
    }
}
