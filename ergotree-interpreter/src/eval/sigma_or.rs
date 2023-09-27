use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::sigma_or::SigmaOr;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::sigma_boolean::cor::Cor;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for SigmaOr {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let items_v_res = self.items.try_mapped_ref(|it| it.eval(env, ctx));
        let items_sigmabool = items_v_res?
            .try_mapped(|it| it.try_extract_into::<SigmaProp>())?
            .mapped(|it| it.value().clone());
        Ok(Value::SigmaProp(Box::new(SigmaProp::new(Cor::normalized(
            items_sigmabool,
        )))))
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
    use std::convert::TryInto;
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;

    use super::*;

    use ergotree_ir::mir::expr::Expr;
    use proptest::collection;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(8))]

        #[test]
        fn eval(sigmaprops in collection::vec(any::<SigmaProp>(), 2..10)) {
            let items = sigmaprops.clone().into_iter().map(|sp| Expr::Const(sp.into())).collect();
            let expr: Expr = SigmaOr::new(items).unwrap().into();
            let ctx = Rc::new(force_any_val::<Context>());
            let res = eval_out::<SigmaProp>(&expr, ctx);
            let expected_sb: Vec<SigmaBoolean> = sigmaprops.into_iter().map(|sp| sp.into()).collect();
            prop_assert!(matches!(res.clone().into(), SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(_))));
            if let SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(Cor {items: actual_sb})) = res.into() {
                prop_assert_eq!(actual_sb, expected_sb.try_into().unwrap());
            }
        }
    }
}
