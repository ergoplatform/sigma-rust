use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for BoolToSigmaProp {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        // JVM v4.x compatibility: pre-v2 ErgoTree scripts (versions V0/V1)
        // accepted a SigmaProp input to BoolToSigmaProp and passed it
        // through unchanged. Mainnet contains historical transactions
        // with `sigmaProp(sigmaProp(...))` (e.g. tx 5fe235558... at
        // block 680,692, address Fo6oijFP2JM87ac7w) that rely on this.
        // The gate is on the SCRIPT's ErgoTree header version, not the
        // block's activated version: a v0 tree spent in a v3+ block
        // still gets the lenient path.
        // See: data/shared/src/main/scala/sigma/ast/trees.scala BoolToSigmaProp.eval
        if ctx.tree_version() < ErgoTreeVersion::V2 {
            if let Value::SigmaProp(sp) = input_v {
                return Ok(Value::SigmaProp(sp));
            }
        }
        let input_v_bool = input_v.try_extract_into::<bool>()?;
        Ok((SigmaProp::new(SigmaBoolean::TrivialProp(input_v_bool))).into())
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::eval::test_util::{eval_out, eval_out_wo_ctx, try_eval_out};
    use core::cell::Cell;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    proptest! {

        #[test]
        fn eval(b in any::<bool>()) {
            let expr: Expr = BoolToSigmaProp {input: Expr::Const(b.into()).into()}.into();
            let res = eval_out_wo_ctx::<SigmaProp>(&expr);
            prop_assert_eq!(res, SigmaProp::new(SigmaBoolean::TrivialProp(b)));
        }
    }

    /// Pre-v2 ErgoTree scripts: BoolToSigmaProp(SigmaProp) → passthrough.
    /// Matches JVM v4.x behavior for historical mainnet scripts.
    #[test]
    fn eval_v0_tree_passes_sigmaprop_through() {
        let inner: Expr = BoolToSigmaProp {
            input: Expr::Const(true.into()).into(),
        }
        .into();
        let outer: Expr = BoolToSigmaProp { input: inner.into() }.into();

        let ctx = force_any_val::<Context>();
        let ctx = Context {
            tree_version: Cell::new(ErgoTreeVersion::V0),
            ..ctx
        };
        let res = eval_out::<SigmaProp>(&outer, &ctx);
        assert_eq!(res, SigmaProp::new(SigmaBoolean::TrivialProp(true)));
    }

    /// V2+ ErgoTree scripts: BoolToSigmaProp(SigmaProp) → strict bool
    /// extraction error. Matches JVM v5+ JIT behavior.
    #[test]
    fn eval_v2_tree_rejects_sigmaprop() {
        let inner: Expr = BoolToSigmaProp {
            input: Expr::Const(true.into()).into(),
        }
        .into();
        let outer: Expr = BoolToSigmaProp { input: inner.into() }.into();

        let ctx = force_any_val::<Context>();
        let ctx = Context {
            tree_version: Cell::new(ErgoTreeVersion::V2),
            ..ctx
        };
        assert!(try_eval_out::<SigmaProp>(&outer, &ctx).is_err());
    }
}
