use alloc::vec::Vec;
use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::mir::xor_of::XorOf;

impl Evaluable for XorOf {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bools = input_v.try_extract_into::<Vec<bool>>()?;
        // JVM v4.x compatibility: pre-v2 ErgoTree scripts (V0/V1) computed
        // xorOf as `distinct.length == 2` — true iff the collection
        // contains BOTH true and false (count and order independent).
        // V2+ uses the correct left-fold XOR (true iff odd number of trues).
        // See: data/shared/src/main/scala/sigma/data/CSigmaDslBuilder.scala
        // (xorOf method, comment "This is buggy version used in v4.x interpreter").
        if ctx.tree_version() < ErgoTreeVersion::V2 {
            let mut has_true = false;
            let mut has_false = false;
            for b in input_v_bools {
                if b { has_true = true; } else { has_false = true; }
                if has_true && has_false { break; }
            }
            return Ok((has_true && has_false).into());
        }
        Ok(input_v_bools.into_iter().fold(false, |a, b| a ^ b).into())
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out;
    use core::cell::Cell;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use proptest::collection;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    fn ctx_with_tree_version(v: ErgoTreeVersion) -> Context<'static> {
        let ctx = force_any_val::<Context>();
        Context {
            tree_version: Cell::new(v),
            ..ctx
        }
    }

    proptest! {

        #[test]
        fn eval_v2_tree_left_fold(bools in collection::vec(any::<bool>(), 0..=10)) {
            // V2+ ErgoTree: proper XOR (true iff odd number of trues).
            let expr: Expr = XorOf {input: Expr::Const(bools.clone().into()).into()}.into();
            let ctx = ctx_with_tree_version(ErgoTreeVersion::V2);
            let res = eval_out::<bool>(&expr, &ctx);
            let expected = bools.into_iter().filter(|x| *x).count() & 1 == 1;
            prop_assert_eq!(res, expected);
        }

        #[test]
        fn eval_v0_tree_distinct_length(bools in collection::vec(any::<bool>(), 0..=10)) {
            // Pre-v2 ErgoTree: JVM v4.x bug — true iff coll contains
            // BOTH true and false (count/order independent).
            let expr: Expr = XorOf {input: Expr::Const(bools.clone().into()).into()}.into();
            let ctx = ctx_with_tree_version(ErgoTreeVersion::V0);
            let res = eval_out::<bool>(&expr, &ctx);
            let has_true = bools.iter().any(|x| *x);
            let has_false = bools.iter().any(|x| !*x);
            prop_assert_eq!(res, has_true && has_false);
        }
    }

    /// Concrete v4.x bug case: [true, true, false] is true under
    /// v4.x bug (both present) but false under v5+ XOR (even count of trues).
    #[test]
    fn eval_v0_tree_two_trues_one_false_is_true() {
        let bools = vec![true, true, false];
        let expr: Expr = XorOf {
            input: Expr::Const(bools.into()).into(),
        }
        .into();
        let ctx = ctx_with_tree_version(ErgoTreeVersion::V0);
        assert_eq!(eval_out::<bool>(&expr, &ctx), true);
    }

    #[test]
    fn eval_v2_tree_two_trues_one_false_is_false() {
        let bools = vec![true, true, false];
        let expr: Expr = XorOf {
            input: Expr::Const(bools.into()).into(),
        }
        .into();
        let ctx = ctx_with_tree_version(ErgoTreeVersion::V2);
        assert_eq!(eval_out::<bool>(&expr, &ctx), false);
    }
}
