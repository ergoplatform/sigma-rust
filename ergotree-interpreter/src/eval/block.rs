use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::val_def::ValDef;
use ergotree_ir::mir::value::Value;
use ergotree_ir::source_span::Spanned;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for BlockValue {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let mut ids = vec![];
        for i in &self.items {
            // TODO: new try_extract_spanned_into?
            let spanned_val_def = &i.clone().try_extract_into::<Spanned<ValDef>>()?;
            let val_def = spanned_val_def.expr();
            let v: Value = val_def.rhs.eval(env, ctx)?;
            env.insert(val_def.id, v);
            ids.push(val_def.id);
        }
        let res = self.result.eval(env, ctx);
        ids.into_iter().for_each(|idx| {
            env.remove(&idx);
        });
        res
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
