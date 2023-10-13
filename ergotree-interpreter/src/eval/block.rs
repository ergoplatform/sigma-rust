use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::val_def::ValDef;
use ergotree_ir::mir::value::Value;
use ergotree_ir::source_span::Spanned;
use hashbrown::HashMap;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for BlockValue {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        // The start of the top-level block of statements does not contain any
        // pre-existing `ValDef`s.
        let is_top_level_block = env.is_empty();

        if is_top_level_block {
            for i in &self.items {
                // TODO: new try_extract_spanned_into?
                let spanned_val_def = &i.clone().try_extract_into::<Spanned<ValDef>>()?;
                let val_def = spanned_val_def.expr();
                let v: Value = val_def.rhs.eval(env, ctx)?;
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
#[cfg(test)]
mod tests {
    use ergotree_ir::mir::block::BlockValue;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(block in any::<BlockValue>()) {
            let e = Expr::BlockValue(block.into());
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
