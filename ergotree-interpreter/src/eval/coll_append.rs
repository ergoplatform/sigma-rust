use ergotree_ir::mir::coll_append::Append;
// use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

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

fn extract_elem_tpe(inp: &Value) -> Result<SType, EvalError> {
    match inp {
        Value::Coll(coll) => Ok(coll.elem_tpe().clone()),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Append: expected Value to be Value::Coll, got: {0:?}",
            inp
        ))),
    }
}

impl Evaluable for Append {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let col2_v = self.col_2.eval(env, ctx)?;
        let input_elem_tpe = extract_elem_tpe(&input_v)?;
        let col2_elem_tpe = extract_elem_tpe(&col2_v)?;
        if input_elem_tpe != col2_elem_tpe {
            return Err(EvalError::UnexpectedValue(format!(
                "Append: expected the same elem tpe, got {0:?} and {1:?}",
                input_elem_tpe, col2_elem_tpe
            )));
        }
        let input_vecval: Vec<Value> = extract_vecval(input_v)?;
        let col_2_vecval: Vec<Value> = extract_vecval(col2_v)?;
        let concat_vecval: Vec<Value> = concat(input_vecval, col_2_vecval);
        Ok(Value::Coll(CollKind::from_vec(
            input_elem_tpe,
            concat_vecval,
        )?))
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::mir::collection::Collection;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::types::stype::SType;

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
            match eval_out_wo_ctx::<Value>(&append_expr) {
                Value::Coll(coll) => assert_eq!(coll.elem_tpe(), &SType::SInt),
                _ => panic!("fail"),
            }
        }
    }

    #[test]
    fn append_byte_array_and_byte() {
        let byte_coll: Constant = vec![1i8, 2i8].into();
        let byte: Expr = Expr::Collection(Collection::new(SType::SByte, vec![3i8.into()]).unwrap());
        let expr: Expr = Expr::Append(Append::new(byte_coll.into(), byte).unwrap());
        assert_eq!(eval_out_wo_ctx::<Vec<i8>>(&expr), vec![1i8, 2, 3]);
    }
}
