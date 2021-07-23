use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::TryExtractFromError;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Collection {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Ok(match self {
            Collection::BoolConstants(bools) => bools.clone().into(),
            Collection::Exprs { elem_tpe, items } => {
                let items_v: Result<Vec<Value>, EvalError> =
                    items.iter().map(|i| i.eval(env, ctx)).collect();
                match elem_tpe {
                    SType::SByte => {
                        let bytes: Result<Vec<i8>, TryExtractFromError> = items_v?
                            .into_iter()
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
mod tests {
    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;

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
}
