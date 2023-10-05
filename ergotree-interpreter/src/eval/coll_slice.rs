use ergotree_ir::mir::coll_slice::Slice;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Slice {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let from_v = self.from.eval(env, ctx)?;
        let until_v = self.until.eval(env, ctx)?;
        let (input_vec, elem_tpe) = match input_v {
            Value::Coll(coll) => Ok((coll.as_vec(), coll.elem_tpe().clone())),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Slice: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        let from = from_v.try_extract_into::<i32>()?;
        let until = until_v.try_extract_into::<i32>()?;
        // intersection of the range with collection bounds
        // to preserve the Scala version semantics of slice op
        // see https://github.com/ergoplatform/sigma-rust/issues/724
        let range = from.max(0) as usize..until.min(input_vec.len() as i32) as usize;
        match input_vec.get(range) {
            Some(slice) => Ok(Value::Coll(CollKind::from_vec(elem_tpe, slice.to_vec())?)),
            // Scala version returns empty collection if the range is out of bounds
            None => Ok(Value::Coll(CollKind::from_vec(elem_tpe, vec![])?)),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::types::stype::SType;

    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;

    #[test]
    fn slice() {
        let expr: Expr = Slice::new(
            Expr::Const(vec![1i64, 2i64, 3i64, 4i64].into()),
            Expr::Const(1i32.into()),
            Expr::Const(3i32.into()),
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<Vec<i64>>(&expr), vec![2i64, 3i64]);

        let expr: Expr = Slice::new(
            Expr::Const(vec![1i64, 2i64, 3i64, 4i64].into()),
            Expr::Const(0i32.into()),
            Expr::Const(4i32.into()),
        )
        .unwrap()
        .into();
        assert_eq!(
            eval_out_wo_ctx::<Vec<i64>>(&expr),
            vec![1i64, 2i64, 3i64, 4i64]
        );
        match eval_out_wo_ctx::<Value>(&expr) {
            Value::Coll(coll) => assert_eq!(coll.elem_tpe(), &SType::SLong),
            _ => panic!("fail"),
        }
    }

    #[test]
    fn slice_empty_coll() {
        // In Scala version the slice with indices out of bounds does not throw
        // but returns an intersection or an empty array.
        // see https://github.com/ergoplatform/sigma-rust/issues/724
        let expr: Expr = Slice::new(
            Expr::Const(Vec::<i64>::new().into()),
            Expr::Const(1i32.into()),
            Expr::Const(3i32.into()),
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<Vec<i64>>(&expr), Vec::<i64>::new());
    }

    #[test]
    fn slice_indices_equal() {
        let expr: Expr = Slice::new(
            Expr::Const(vec![1i64, 2i64, 3i64, 4i64].into()),
            Expr::Const(1i32.into()),
            Expr::Const(1i32.into()),
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<Vec<i64>>(&expr), Vec::<i64>::new());
    }

    #[test]
    fn slice_start_index_greater_than_end_index() {
        // In Scala version the slice with indices out of bounds does not throw
        // but returns an intersection or an empty array.
        // see https://github.com/ergoplatform/sigma-rust/issues/724
        let expr: Expr = Slice::new(
            Expr::Const(vec![1i64, 2i64, 3i64, 4i64].into()),
            Expr::Const(3i32.into()),
            Expr::Const(1i32.into()),
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<Vec<i64>>(&expr), Vec::<i64>::new());
    }

    #[test]
    fn slice_index_out_of_bounds() {
        // In Scala version the slice with indices out of bounds does not throw
        // but returns an intersection or an empty array.
        // see https://github.com/ergoplatform/sigma-rust/issues/724
        let expr: Expr = Slice::new(
            Expr::Const(vec![1i64, 2i64, 3i64, 4i64].into()),
            Expr::Const((-1i32).into()),
            Expr::Const(1i32.into()),
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<Vec<i64>>(&expr), vec![1i64]);

        let expr: Expr = Slice::new(
            Expr::Const(vec![1i64, 2i64, 3i64, 4i64].into()),
            Expr::Const(0i32.into()),
            Expr::Const(5i32.into()),
        )
        .unwrap()
        .into();
        assert_eq!(
            eval_out_wo_ctx::<Vec<i64>>(&expr),
            vec![1i64, 2i64, 3i64, 4i64]
        );

        let expr: Expr = Slice::new(
            Expr::Const(vec![1i64, 2i64, 3i64, 4i64].into()),
            Expr::Const(9i32.into()),
            Expr::Const(10i32.into()),
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<Vec<i64>>(&expr), Vec::<i64>::new());
    }
}
