use alloc::sync::Arc;

use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::TryExtractFromError;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Collection {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(20)?; // ConcreteCollection = Fixed(20)
        Ok(match self {
            Collection::BoolConstants(bools) => {
                // The JVM models a boolean-constant collection as N
                // BooleanConstant nodes: ConcreteCollection.eval charges the
                // Fixed(20) above, then evaluates each item, charging
                // Constant.costKind = FixedCost(JitCost(5)) per element (sigma
                // `values.scala` ConcreteCollection.eval per-item `evalTo` +
                // Constant.costKind). The `Exprs` arm below already pays this
                // via each `Expr::Const` eval (`expr.rs`: Constant = Fixed(5)),
                // but the packed-bool form converts directly, so charge the
                // equivalent N * 5 here to match the JVM (total = 20 + 5n).
                ctx.add_jit_cost(5 * bools.len() as u64)?;
                bools.clone().into()
            }
            Collection::Exprs { elem_tpe, items } => {
                let items_v: Result<Arc<[Value]>, EvalError> =
                    items.iter().map(|i| i.eval(env, ctx)).collect();
                match elem_tpe {
                    SType::SByte => {
                        let bytes: Result<Arc<[i8]>, TryExtractFromError> = items_v?
                            .iter()
                            .cloned()
                            .map(|i| i.try_extract_into::<i8>())
                            .collect();
                        Value::Coll(CollKind::NativeColl(NativeColl::CollByte(bytes?)))
                    }
                    _ => Value::Coll(CollKind::WrappedColl {
                        elem_tpe: elem_tpe.clone(),
                        items: items_v?,
                    }),
                }
            }
        })
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::eval::test_util::{eval_out, eval_out_wo_ctx};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    proptest! {

        #[test]
        fn eval_byte_coll(bytes in any::<Vec<i8>>()) {
            let value: Value = bytes.clone().into();
            let exprs: Vec<Expr> = bytes.into_iter().map(|b| Expr::Const(b.into())).collect();
            let coll: Expr = Collection::new(SType::SByte, exprs).unwrap().into();
            let res = eval_out_wo_ctx::<Value>(&coll);
            prop_assert_eq!(res, value);
        }

        #[test]
        fn eval_bool_coll(bools in any::<Vec<bool>>()) {
            let exprs: Vec<Expr> = bools.clone().into_iter().map(|b| Expr::Const(b.into())).collect();
            let coll: Expr = Collection::new(SType::SBoolean, exprs).unwrap().into();
            let res = eval_out_wo_ctx::<Vec<bool>>(&coll);
            prop_assert_eq!(res, bools);
        }

        #[test]
        fn eval_long_coll(longs in any::<Vec<i64>>()) {
            let exprs: Vec<Expr> = longs.clone().into_iter().map(|b| Expr::Const(b.into())).collect();
            let coll: Expr = Collection::new(SType::SLong, exprs).unwrap().into();
            let res = eval_out_wo_ctx::<Vec<i64>>(&coll);
            prop_assert_eq!(res, longs);
        }

        #[test]
        fn eval_bytes_coll_coll(bb in any::<Vec<Vec<i8>>>()) {
            let exprs: Vec<Expr> = bb.clone().into_iter().map(|b| Expr::Const(b.into())).collect();
            let coll: Expr = Collection::new(SType::SColl(SType::SByte.into()), exprs).unwrap().into();
            let res = eval_out_wo_ctx::<Vec<Vec<i8>>>(&coll);
            prop_assert_eq!(res, bb);
        }
    }

    #[test]
    fn bool_constants_coll_charges_per_constant() {
        // ConcreteCollection = Fixed(20); each boolean constant is a
        // BooleanConstant = Fixed(5) (sigma `values.scala` :878 / :380). The
        // packed `BoolConstants` form must charge 20 + 5n to match the JVM and
        // the `Exprs` form. Cross-validated by the santa eval fixture
        // `coll_bool_constants_3` (tree 00850305) -> JVM 35.
        let cost = |n: usize| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let exprs: Vec<Expr> = (0..n).map(|i| Expr::Const((i % 2 == 0).into())).collect();
            let coll: Expr = Collection::new(SType::SBoolean, exprs).unwrap().into();
            let _: Vec<bool> = eval_out(&coll, &ctx);
            ctx.jit_cost_value() - before
        };
        assert_eq!(cost(0), 20); // empty: 20 + 0
        assert_eq!(cost(3), 35); // 20 + 3*5  (coll_bool_constants_3)
        assert_eq!(cost(5), 45); // 20 + 5*5
    }
}
