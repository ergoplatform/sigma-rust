use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::val_def::ValDef;
use ergotree_ir::mir::value::Value;
use ergotree_ir::source_span::Spanned;
use hashbrown::HashMap;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for BlockValue {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_per_item_jit_cost(1, 1, 10, self.items.len() as u32)?;
        // The start of the top-level block of statements does not contain any
        // pre-existing `ValDef`s.
        let is_top_level_block = env.is_empty();

        if is_top_level_block {
            for i in &self.items {
                // TODO: new try_extract_spanned_into?
                let spanned_val_def = &i.clone().try_extract_into::<Spanned<ValDef>>()?;
                let val_def = spanned_val_def.expr();
                let v: Value = val_def.rhs.eval(env, ctx)?;
                ctx.add_jit_cost(crate::eval::ADD_TO_ENV_COST)?;
                env.insert(val_def.id, v);
            }
            // Keep all `ValDef`s introduced in this block
            self.result.eval(env, ctx)
        } else {
            let mut existing_variables = HashMap::new();
            let mut new_variables = vec![];

            for i in &self.items {
                // TODO: new try_extract_spanned_into?
                let spanned_val_def = &i.clone().try_extract_into::<Spanned<ValDef>>()?;
                let val_def = spanned_val_def.expr();
                let idx = val_def.id;
                let v: Value = val_def.rhs.eval(env, ctx)?;
                ctx.add_jit_cost(crate::eval::ADD_TO_ENV_COST)?;
                if let Some(old_val) = env.get(idx) {
                    existing_variables.insert(idx, old_val.clone());
                } else {
                    new_variables.push(idx);
                }
                env.insert(idx, v);
            }
            let res = self.result.eval(env, ctx);
            new_variables.into_iter().for_each(|idx| {
                env.remove(&idx);
            });
            existing_variables
                .into_iter()
                .for_each(|(idx, orig_value)| {
                    env.insert(idx, orig_value);
                });
            res
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergotree_ir::mir::block::BlockValue;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::val_def::ValDef;
    use ergotree_ir::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(block in any::<BlockValue>()) {
            let e = Expr::BlockValue(block.into());
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }

    // Parity gap: Scala charges ADD_TO_ENV_COST (5 JIT) per ValDef inserted
    // into the interpreter env. Pre-fix, sigma-rust only paid BlockValue's
    // per-item base (1 + ceil(n/10)) and the rhs eval cost, so every ValDef
    // undercounted by 5 JIT vs the Scala reference. Observed on mainnet tx
    // 518acec… @ 700032: 4 ValDefs × 5 = 20 JIT short on block cost.
    #[test]
    fn block_value_charges_add_to_env_per_val_def() {
        use crate::eval::test_util::try_eval_out;
        use alloc::boxed::Box;
        use ergotree_ir::chain::context::Context;
        use sigma_test_util::force_any_val;

        let run = |n_val_defs: u32| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let items: alloc::vec::Vec<Expr> = (1..=n_val_defs)
                .map(|i| {
                    ValDef {
                        id: i.into(),
                        rhs: Box::new(Expr::Const((i as i32).into())),
                    }
                    .into()
                })
                .collect();
            let expr: Expr = Expr::BlockValue(
                BlockValue {
                    items,
                    result: Box::new(Expr::Const(0i32.into())),
                }
                .into(),
            );
            let _: i32 = try_eval_out(&expr, &ctx).unwrap();
            ctx.jit_cost_value() - before
        };

        // Both 1 and 4 ValDefs fall in the same BlockValue chunk (chunk_size=10),
        // so the block base cost is identical; per-ValDef cost = rhs Const (5) +
        // ADD_TO_ENV_COST (5) = 10. Delta (4 - 1) × 10 = 30 post-fix.
        // Pre-fix (no ADD_TO_ENV) would give delta 3 × 5 = 15.
        let delta_1 = run(1);
        let delta_4 = run(4);
        assert_eq!(
            delta_4 - delta_1,
            30,
            "BlockValue must charge ADD_TO_ENV_COST (5 JIT) per ValDef on top \
             of the rhs Const eval (5); got {} JIT delta between 4-valdef and \
             1-valdef blocks, expected 30.",
            delta_4 - delta_1,
        );
    }
}
