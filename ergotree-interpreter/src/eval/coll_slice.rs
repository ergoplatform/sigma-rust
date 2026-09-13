use ergotree_ir::mir::coll_slice::Slice;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Slice {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
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
        // Scala charges based on requested range size (max(0, until - from)),
        // not input length or clipped output length — costing is pre-clipping.
        let n_items = 0i32.max(until - from) as u32;
        ctx.add_per_item_jit_cost(10, 2, 100, n_items)?;
        // intersection of the range with collection bounds
        // to preserve the Scala version semantics of slice op
        // see https://github.com/ergoplatform/sigma-rust/issues/724
        let range = from.max(0) as usize..until.min(input_vec.len() as i32) as usize;
        match input_vec.get(range) {
            Some(slice) => Ok(Value::Coll(CollKind::from_collection(elem_tpe, slice)?)),
            // Scala version returns empty collection if the range is out of bounds
            None => Ok(Value::Coll(CollKind::from_collection(elem_tpe, [])?)),
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
    use crate::eval::test_util::eval_out_wo_ctx;

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

    // Bug 7 regression: Slice cost must scale with the requested range
    // (max(0, until - from)) per Scala semantics, not with the input collection
    // length. Pre-fix, sigma-rust charged based on `input_vec.len()`, so
    // slicing a tiny window from a huge collection was overpriced (and a tx
    // building a giant intermediate just to slice 1 element could exceed the
    // cost limit even though the requested work was trivial).
    #[test]
    fn slice_charges_output_not_input_size() {
        use ergotree_ir::chain::context::Context;
        use sigma_test_util::force_any_val;

        use crate::eval::test_util::try_eval_out;

        let run = |coll: Vec<i64>, from: i32, until: i32| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let expr: Expr = Slice::new(
                Expr::Const(coll.into()),
                Expr::Const(from.into()),
                Expr::Const(until.into()),
            )
            .unwrap()
            .into();
            let _: Vec<i64> = try_eval_out(&expr, &ctx).unwrap();
            ctx.jit_cost_value() - before
        };

        // Same requested range (until - from = 2), wildly different input sizes.
        // Const eval is fixed (5 JIT regardless of payload), so the only way
        // the deltas can diverge is if Slice's per-item charge is reading
        // input length — the bug.
        let small_input = run((0..5i64).collect(), 0, 2);
        let large_input = run((0..1000i64).collect(), 0, 2);
        assert_eq!(
            small_input, large_input,
            "Slice cost must depend on (until - from), not input length. \
             Got {} JIT for 5-elem input vs {} JIT for 1000-elem input.",
            small_input, large_input,
        );

        // And the requested range MUST drive the cost: a larger requested range
        // (200 items, even though clipped to 5) is more expensive than a
        // 2-item range. n_items = max(0, 200 - 0) = 200 → 2 chunks of 100;
        // n_items = max(0, 2 - 0) = 2 → 1 chunk. Per-chunk cost = 2 JIT.
        let large_range = run((0..5i64).collect(), 0, 200);
        assert!(
            large_range > small_input,
            "Slice cost must scale with requested range. small_input={}, \
             large_range={}",
            small_input,
            large_range,
        );
    }
}
