use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use alloc::vec::Vec;
use core::convert::TryFrom;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::subst_const::SubstConstants;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaSerializable;
use sigma_util::AsVecI8;
use sigma_util::AsVecU8;

impl Evaluable for SubstConstants {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let script_bytes_v = self.script_bytes.eval(env, ctx)?;
        let positions_v = self.positions.eval(env, ctx)?;
        let new_values_v = self.new_values.eval(env, ctx)?;

        let positions: Vec<usize> = positions_v
            .try_extract_into::<Vec<i32>>()?
            .into_iter()
            .map(|i| i as usize)
            .collect();

        let new_constants = if let Value::Coll(CollKind::WrappedColl { items, .. }) = new_values_v {
            let mut items_const = vec![];
            for v in &*items {
                let c = Constant::try_from(v.to_static()).map_err(EvalError::Misc)?;
                items_const.push(c);
            }
            items_const
        } else {
            return Err(EvalError::Misc(format!(
                "SubstConstants: expected evaluation of `new_values` be of type `Coll[_]`, got \
                    {:?} instead",
                new_values_v
            )));
        };

        if new_constants.len() != positions.len() {
            return Err(EvalError::Misc(format!(
                "SubstConstants: `positions.len()` (== {}) and `new_values.len()` (== {}) differ",
                positions.len(),
                new_constants.len()
            )));
        }

        if let Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b))) = script_bytes_v {
            // Substitue constants with repeated calls to `ErgoTree::with_constant`.
            let mut ergo_tree = ErgoTree::sigma_parse_bytes(&b.as_vec_u8())?;
            let num_constants = ergo_tree.constants_len().map_err(to_misc_err)?;
            // Charge based on the template's constants count, matching Scala's
            // `ErgoTreeSerializer.substituteConstants` — the walk visits every
            // constant in the template regardless of how many positions the
            // caller asks to replace.
            ctx.add_per_item_jit_cost(100, 100, 1, num_constants as u32)?;
            for (ix, i) in positions.iter().enumerate() {
                if *i < num_constants {
                    ergo_tree = ergo_tree
                        .with_constant(*i, new_constants[ix].clone())
                        .map_err(to_misc_err)?;
                } else {
                    return Err(EvalError::Misc(format!(
                        "SubstConstants: positions[{}] == {} is an out of bound index with \
                       respect to the serialized ErgoTree's constant list",
                        ix, *i
                    )));
                }
            }
            Ok(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(
                ergo_tree.sigma_serialize_bytes()?.as_vec_i8().into(),
            ))))
        } else {
            Err(EvalError::Misc(format!(
                "SubstConstants: expected evaluation of `script_bytes` to be of type `Coll[SBytes]`, \
                 got {:?} instead",
                script_bytes_v
            )))
        }
    }
}

fn to_misc_err<T: core::fmt::Debug>(e: T) -> EvalError {
    EvalError::Misc(format!("{:?}", e))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[allow(clippy::expect_used)]
#[allow(clippy::unreachable)]
mod tests {
    use ergotree_ir::{
        ergo_tree::{ErgoTree, ErgoTreeHeader},
        mir::{
            bin_op::{ArithOp, BinOp, BinOpKind},
            expr::Expr,
            value::StoreWrapped,
        },
        serialization::SigmaSerializable,
        types::stype::LiftIntoSType,
    };
    use proptest::prelude::*;

    use crate::eval::test_util::try_eval_out_wo_ctx;

    use super::*;
    proptest! {

        #[test]
        fn eval_single_substitution(original in any::<((i32, i32), Vec<i64>)>(), new in any::<((i32, i32), Vec<i64>)>()) {
            test_single_substitution(original, new);
        }

        #[test]
        fn eval_3_substitutions(original in any::<(i32, i32, i32)>(), new in any::<(i32, i32, i32)>()) {
            test_3_substitutions(original, new);
        }
    }

    fn test_single_substitution<T: Clone + LiftIntoSType + StoreWrapped + Into<Constant>>(
        original: T,
        new: T,
    ) {
        let expr = Expr::Const(original.clone().into());
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr).unwrap();
        assert_eq!(ergo_tree.constants_len().unwrap(), 1);
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), original.into());

        let script_bytes =
            Expr::Const(Constant::from(ergo_tree.sigma_serialize_bytes().unwrap())).into();
        let positions = Expr::Const(Constant::from(vec![0])).into();
        let new_values = Expr::Const(Constant::from(vec![new.clone()])).into();

        let subst_const = Expr::SubstConstants(
            SubstConstants {
                script_bytes,
                positions,
                new_values,
            }
            .into(),
        );

        let x: Value = try_eval_out_wo_ctx(&subst_const).unwrap();
        if let Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b))) = x {
            let new_ergo_tree = ErgoTree::sigma_parse_bytes(&b.as_vec_u8()).unwrap();
            assert_eq!(new_ergo_tree.constants_len().unwrap(), 1);
            assert_eq!(new_ergo_tree.get_constant(0).unwrap().unwrap(), new.into());
        } else {
            unreachable!();
        }
    }

    fn test_3_substitutions(original: (i32, i32, i32), new: (i32, i32, i32)) {
        let (o0, o1, o2) = original;
        let (n0, n1, n2) = new;
        let expr = Expr::BinOp(
            BinOp {
                kind: BinOpKind::Arith(ArithOp::Plus),
                left: Box::new(Expr::Const(o0.into())),
                right: Box::new(Expr::BinOp(
                    BinOp {
                        kind: BinOpKind::Arith(ArithOp::Multiply),
                        left: Box::new(Expr::Const(o1.into())),
                        right: Box::new(Expr::Const(o2.into())),
                    }
                    .into(),
                )),
            }
            .into(),
        );
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr).unwrap();
        assert_eq!(ergo_tree.constants_len().unwrap(), 3);
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), o0.into());
        assert_eq!(ergo_tree.get_constant(1).unwrap().unwrap(), o1.into());
        assert_eq!(ergo_tree.get_constant(2).unwrap().unwrap(), o2.into());

        let script_bytes =
            Expr::Const(Constant::from(ergo_tree.sigma_serialize_bytes().unwrap())).into();

        let positions = Expr::Const(Constant::from(vec![1, 2, 0])).into();

        let new_values = Expr::Const(Constant::from(vec![n0, n1, n2])).into();

        let subst_const = Expr::SubstConstants(
            SubstConstants {
                script_bytes,
                positions,
                new_values,
            }
            .into(),
        );

        let x: Value = try_eval_out_wo_ctx(&subst_const).unwrap();
        if let Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b))) = x {
            let new_ergo_tree = ErgoTree::sigma_parse_bytes(&b.as_vec_u8()).unwrap();
            assert_eq!(new_ergo_tree.constants_len().unwrap(), 3);
            assert_eq!(new_ergo_tree.get_constant(0).unwrap().unwrap(), n2.into());
            assert_eq!(new_ergo_tree.get_constant(1).unwrap().unwrap(), n0.into());
            assert_eq!(new_ergo_tree.get_constant(2).unwrap().unwrap(), n1.into());
        } else {
            unreachable!();
        }
    }

    // Bug 3 regression: SubstConstants cost must be charged based on the
    // template tree's `constants_len()`, not on the caller-supplied positions
    // count. Build one 3-constant template and run substitution twice with
    // different position counts — after the fix, total JIT cost must be
    // identical since the template walk is the same. Pre-fix, the two runs
    // diverged because the charge scaled with positions.len().
    #[test]
    #[allow(clippy::identity_op)]
    fn subst_const_cost_uses_template_count_not_positions_len() {
        use crate::eval::test_util::try_eval_out;
        use ergotree_ir::chain::context::Context;
        use sigma_test_util::force_any_val;

        // Template tree with 3 constants (a + b * c).
        let expr = Expr::BinOp(
            BinOp {
                kind: BinOpKind::Arith(ArithOp::Plus),
                left: Box::new(Expr::Const(1i32.into())),
                right: Box::new(Expr::BinOp(
                    BinOp {
                        kind: BinOpKind::Arith(ArithOp::Multiply),
                        left: Box::new(Expr::Const(2i32.into())),
                        right: Box::new(Expr::Const(3i32.into())),
                    }
                    .into(),
                )),
            }
            .into(),
        );
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr).unwrap();
        assert_eq!(ergo_tree.constants_len().unwrap(), 3);
        let tree_bytes = ergo_tree.sigma_serialize_bytes().unwrap();
        let script_bytes_expr: Expr = Expr::Const(Constant::from(tree_bytes));

        let run = |positions: Vec<i32>, new_vals: Vec<i32>| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let script_bytes: Box<Expr> = Box::new(script_bytes_expr.clone());
            let positions_expr: Box<Expr> = Box::new(Expr::Const(Constant::from(positions)));
            let new_values_expr: Box<Expr> = Box::new(Expr::Const(Constant::from(new_vals)));
            let subst = Expr::SubstConstants(
                SubstConstants {
                    script_bytes,
                    positions: positions_expr,
                    new_values: new_values_expr,
                }
                .into(),
            );
            let _: Value = try_eval_out(&subst, &ctx).unwrap();
            ctx.jit_cost_value() - before
        };

        let delta_1_position = run(vec![0], vec![999]);
        let delta_3_positions = run(vec![0, 1, 2], vec![999, 888, 777]);

        // Both runs walk the same 3-constant template, so SubstConstants-specific
        // cost must be identical. `Expr::Const` eval is fixed (5 JIT) regardless
        // of payload size, so the Const arms used to feed positions/new_values
        // don't contribute differently. Any non-zero difference indicates the
        // old positions.len()-based charge leaked back in.
        assert_eq!(
            delta_1_position, delta_3_positions,
            "SubstConstants cost must depend on template's constants_len (3), \
             not on positions.len() (1 vs 3). Deltas diverged: {} vs {}",
            delta_1_position, delta_3_positions,
        );
    }
}
