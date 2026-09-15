use ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for SigmaPropBytes {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::SigmaProp(sigma_prop) => {
                // Cost scales with the proposition's node count, matching
                // Scala's `SigmaPropBytes.eval` (`numNodes = wrappedValue.size`,
                // PerItemCost(35, 6, 1)), charged after the input eval as the
                // JVM does. Previously hardcoded n=1, undercharging every
                // multi-node conjecture (and ProveDHTuple, whose size is 4).
                ctx.add_per_item_jit_cost(35, 6, 1, sigma_prop.value().size() as u32)?;
                Ok(sigma_prop.prop_bytes()?.into())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected SigmaPropBytes input to be Value::SigmaProp, got {0:?}",
                input_v
            ))),
        }
    }
}

#[cfg(feature = "arbitrary")]
#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out_wo_ctx;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(8))]

        #[test]
        fn eval(v in any::<SigmaProp>()) {
            let expected_bytes = v.prop_bytes().unwrap();
            let input: Constant = v.into();
            let e: Expr = SigmaPropBytes {
                input: Box::new(input.into()),
            }
            .into();
            prop_assert_eq!(eval_out_wo_ctx::<Vec<u8>>(&e), expected_bytes);
        }
    }

    // propBytes cost is PerItemCost(35, 6, 1) over the proposition's node
    // count (= 35 + 6*size), matching Scala. A ProveDlog (size 1) and a
    // ProveDHTuple (size 4) are both single SigmaProp consts with the same
    // input-eval cost, so the cost delta isolates the node-count scaling:
    // 6*(4-1) = 18. Pre-fix (hardcoded n=1) the delta was 0.
    #[test]
    fn propbytes_cost_scales_with_node_count() {
        use crate::eval::test_util::try_eval_out;
        use ergotree_ir::chain::context::Context;
        use ergotree_ir::sigma_protocol::sigma_boolean::{ProveDhTuple, ProveDlog, SigmaBoolean};
        use sigma_test_util::force_any_val;

        let cost_of = |sp: SigmaProp| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let input: Constant = sp.into();
            let e: Expr = SigmaPropBytes {
                input: Box::new(input.into()),
            }
            .into();
            let _: Vec<u8> = try_eval_out(&e, &ctx).unwrap();
            ctx.jit_cost_value() - before
        };

        let dlog: SigmaProp = SigmaProp::new(SigmaBoolean::from(force_any_val::<ProveDlog>()));
        let dht: SigmaProp = SigmaProp::new(SigmaBoolean::from(force_any_val::<ProveDhTuple>()));
        let delta = cost_of(dht) - cost_of(dlog);
        assert_eq!(
            delta, 18,
            "propBytes cost must scale with node count (ProveDHTuple size 4 vs \
             ProveDlog size 1 = 6*(4-1) = 18); got {}",
            delta,
        );
    }
}
