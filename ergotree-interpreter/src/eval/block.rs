use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::val_def::{ValDef, ValId};
use ergotree_ir::mir::value::Value;
use ergotree_ir::source_span::Spanned;
use hashbrown::HashMap;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

/// Evaluate a top-level `BlockValue` while retaining its own `ValDef` bindings
/// in `env` for `ReductionDiagnosticInfo` (nested `BlockValue`s still restore
/// `env` via `BlockValue::eval`).
pub(crate) fn eval_keeping_env<'ctx>(
    block: &BlockValue,
    env: &mut Env<'ctx>,
    ctx: &Context<'ctx>,
) -> Result<Value<'ctx>, EvalError> {
    for i in &block.items {
        // TODO: new try_extract_spanned_into?
        let spanned_val_def = &i.clone().try_extract_into::<Spanned<ValDef>>()?;
        let val_def = spanned_val_def.expr();
        let v: Value = val_def.rhs.eval(env, ctx)?;
        env.insert(val_def.id, v);
    }
    block.result.eval(env, ctx)
}

impl Evaluable for BlockValue {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let mut saved: HashMap<ValId, Option<Value<'ctx>>> = HashMap::new();
        let res = (|| -> Result<Value<'ctx>, EvalError> {
            for i in &self.items {
                // TODO: new try_extract_spanned_into?
                let spanned_val_def = &i.clone().try_extract_into::<Spanned<ValDef>>()?;
                let val_def = spanned_val_def.expr();
                let idx = val_def.id;
                let v: Value = val_def.rhs.eval(env, ctx)?;
                saved.entry(idx).or_insert_with(|| env.get(idx).cloned());
                env.insert(idx, v);
            }
            self.result.eval(env, ctx)
        })();
        for (idx, orig_value) in saved {
            match orig_value {
                Some(v) => env.insert(idx, v),
                None => env.remove(&idx),
            }
        }
        res
    }
}

#[allow(clippy::panic, clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use ergotree_ir::mir::bin_op::{ArithOp, BinOp, BinOpKind, RelationOp};
    use ergotree_ir::mir::block::BlockValue;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::val_def::ValDef;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::serialization::sigma_serialize_roundtrip;
    use ergotree_ir::types::stype::SType;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    use crate::eval::env::Env;
    use crate::eval::test_util::{eval_out_wo_ctx, try_eval_out_wo_ctx};
    use crate::eval::{Context, EvalError, Evaluable};

    fn val_def(id: u32, rhs: Expr) -> Expr {
        ValDef {
            id: id.into(),
            rhs: Box::new(rhs),
        }
        .into()
    }

    fn val_use_int(id: u32) -> Expr {
        ValUse {
            val_id: id.into(),
            tpe: SType::SInt,
        }
        .into()
    }

    fn plus(left: Expr, right: Expr) -> Expr {
        BinOp {
            kind: BinOpKind::Arith(ArithOp::Plus),
            left: Box::new(left),
            right: Box::new(right),
        }
        .into()
    }

    fn eq(left: Expr, right: Expr) -> Expr {
        BinOp {
            kind: BinOpKind::Relation(RelationOp::Eq),
            left: Box::new(left),
            right: Box::new(right),
        }
        .into()
    }

    fn unwrap_spanned(err: &EvalError) -> &EvalError {
        match err {
            EvalError::Spanned(s) => unwrap_spanned(&s.error),
            other => other,
        }
    }

    #[test]
    fn sibling_blocks_at_empty_env_do_not_leak_val_defs() {
        // ({ val v1 = 1; v1 }) == ({ val v2 = 2; v1 })
        // Left sibling enters with an empty Env; v1 must not leak to the right sibling.
        let left_block: Expr = BlockValue {
            items: vec![val_def(1, 1i32.into())],
            result: Box::new(val_use_int(1)),
        }
        .into();
        let right_block: Expr = BlockValue {
            items: vec![val_def(2, 2i32.into())],
            result: Box::new(val_use_int(1)),
        }
        .into();
        let expr = eq(left_block, right_block);
        let roundtripped = sigma_serialize_roundtrip(&expr);
        let err = try_eval_out_wo_ctx::<bool>(&roundtripped).unwrap_err();
        assert!(
            matches!(unwrap_spanned(&err), EvalError::NotFound(_)),
            "expected unbound ValUse(1) in sibling block, got {:?}",
            err
        );
    }

    #[test]
    fn duplicate_val_id_shadowing_restores_outer_binding() {
        // { val v1 = 5; ({ val v1 = 10; val v1 = v1 + 10; v1 } + v1) == 25 }
        // Inner block shadows v1 twice; outer v1 must be restored to 5 (not 10).
        let inner_block: Expr = BlockValue {
            items: vec![
                val_def(1, 10i32.into()),
                val_def(1, plus(val_use_int(1), 10i32.into())),
            ],
            result: Box::new(val_use_int(1)),
        }
        .into();
        let outer_block: Expr = BlockValue {
            items: vec![val_def(1, 5i32.into())],
            result: Box::new(eq(plus(inner_block, val_use_int(1)), 25i32.into())),
        }
        .into();
        let roundtripped = sigma_serialize_roundtrip(&outer_block);
        assert!(eval_out_wo_ctx::<bool>(&roundtripped));
    }

    #[test]
    fn duplicate_fresh_val_id_in_nested_block_does_not_escape() {
        // { val v9 = 1; { val v1 = 10; val v1 = 20; v1 } + v1 }
        // v1 is not bound in the outer block; rebinding v1 twice inside the inner
        // block must remove v1 on exit instead of re-inserting 10.
        let inner_block: Expr = BlockValue {
            items: vec![val_def(1, 10i32.into()), val_def(1, 20i32.into())],
            result: Box::new(val_use_int(1)),
        }
        .into();
        let outer_block: Expr = BlockValue {
            items: vec![val_def(9, 1i32.into())],
            result: Box::new(plus(inner_block, val_use_int(1))),
        }
        .into();
        let roundtripped = sigma_serialize_roundtrip(&outer_block);
        let err = try_eval_out_wo_ctx::<i32>(&roundtripped).unwrap_err();
        assert!(
            matches!(unwrap_spanned(&err), EvalError::NotFound(_)),
            "expected unbound ValUse(1) after nested block, got {:?}",
            err
        );
    }

    #[test]
    fn error_mid_block_restores_env() {
        // { val v1 = 10; val v3 = 99; val v2 = 1 / 0; v1 } with outer v1 = 5 and v2/v3 absent.
        let div_by_zero: Expr = BinOp {
            kind: BinOpKind::Arith(ArithOp::Divide),
            left: Box::new(1i32.into()),
            right: Box::new(0i32.into()),
        }
        .into();
        let block = BlockValue {
            items: vec![
                val_def(1, 10i32.into()),
                val_def(3, 99i32.into()),
                val_def(2, div_by_zero),
            ],
            result: Box::new(val_use_int(1)),
        };
        let ctx = force_any_val::<Context>();
        let mut env = Env::empty();
        env.insert(1.into(), 5i32.into());
        let err = block.eval(&mut env, &ctx).unwrap_err();
        assert!(
            matches!(unwrap_spanned(&err), EvalError::ArithmeticException(_)),
            "expected ArithmeticException, got {:?}",
            err
        );
        assert_eq!(env.get(1.into()), Some(&5i32.into()));
        assert_eq!(env.get(2.into()), None);
        assert_eq!(env.get(3.into()), None);
    }

    #[test]
    fn reduce_to_crypto_root_block_non_valdef_returns_spanned_err() {
        // Wire-deserializable root BlockValue containing a non-ValDef item must return
        // Err(EvalError::SpannedWithSource(_)) in reduce_to_crypto rather than panicking
        // in EvalError::wrap_spanned_with_src.
        use ergotree_ir::ergo_tree::ErgoTree;

        let root_expr: Expr = BlockValue {
            items: vec![Expr::Const(1i32.into())],
            result: Box::new(Expr::Const(true.into())),
        }
        .into();
        let tree: ErgoTree = root_expr.try_into().unwrap();
        let ctx = force_any_val::<Context>();
        let err = crate::eval::reduce_to_crypto(&tree, &ctx).unwrap_err();
        assert!(
            matches!(err, EvalError::SpannedWithSource(_)),
            "expected SpannedWithSource error, got {:?}",
            err
        );
    }

    proptest! {

        #[test]
        fn ser_roundtrip(block in any::<BlockValue>()) {
            let e = Expr::BlockValue(block.into());
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
