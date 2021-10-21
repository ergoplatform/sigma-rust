use std::convert::TryInto;

use ergotree_ir::mir::atleast::Atleast;
use ergotree_ir::mir::constant::TryExtractFromError;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Atleast {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let bound_v = self.bound.eval(env, ctx)?;
        let input_v = self.input.eval(env, ctx)?;

        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Atleast: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;

        let bound = bound_v.try_extract_into::<i32>()?;
        let input = normalized_input_vals
            .into_iter()
            .map(|i| {
                i.try_extract_into::<SigmaProp>()
                    .map(|sp| sp.value().clone())
            })
            .collect::<Result<Vec<SigmaBoolean>, TryExtractFromError>>()?;

        Ok(Value::SigmaProp(Box::new(SigmaProp::new(
            Cthreshold::reduce(bound, input.try_into()?).into(),
        ))))
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::value::CollKind;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
    use ergotree_ir::types::stype::SType;
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
        fn eval(sigmaprops in collection::vec(any::<SigmaProp>(), 2..4)) {
            let items: Value = CollKind::from_vec(SType::SSigmaProp,
                sigmaprops.into_iter().map(|s| s.into()).collect::<Vec<Value>>()).unwrap().into();
            let expr: Expr = Atleast::new(1i32.into(),
                Constant {tpe: SType::SColl(SType::SSigmaProp.into()), v: items}.into()).unwrap().into();
            let ctx = Rc::new(force_any_val::<Context>());
            let res = eval_out::<SigmaProp>(&expr, ctx);
            prop_assert!(matches!(res.into(),
                SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(_))));
        }
    }
}
