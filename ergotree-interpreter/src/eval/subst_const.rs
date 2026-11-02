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
            // Byte-level substitution mirroring sigma-state's
            // `ErgoTreeSerializer.substituteConstants`: the tree body is never
            // parsed and out-of-range positions are a no-op, so a malformed
            // body or an OOB position returns the original bytes (JVM parity)
            // instead of erroring.
            let (new_bytes, _num_constants) = ErgoTree::substitute_constants(
                b.as_vec_u8(),
                &positions,
                &new_constants,
                ctx.tree_version(),
            )
            .map_err(to_misc_err)?;
            Ok(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(
                new_bytes.as_vec_i8().into(),
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
            // The template's root is not a `SigmaProp`, which a tree parse rejects (rule
            // 1001) but SubstConstants never parses: compare the bytes with the tree built
            // from the new constant.
            let expected = ErgoTree::new(ErgoTreeHeader::v0(true), &Expr::Const(new.into()))
                .unwrap()
                .sigma_serialize_bytes()
                .unwrap();
            assert_eq!(b.as_vec_u8(), expected);
        } else {
            unreachable!();
        }
    }

    /// `a + b * c`, three `Int` constants
    fn plus_times(a: i32, b: i32, c: i32) -> Expr {
        Expr::BinOp(
            BinOp {
                kind: BinOpKind::Arith(ArithOp::Plus),
                left: Box::new(Expr::Const(a.into())),
                right: Box::new(Expr::BinOp(
                    BinOp {
                        kind: BinOpKind::Arith(ArithOp::Multiply),
                        left: Box::new(Expr::Const(b.into())),
                        right: Box::new(Expr::Const(c.into())),
                    }
                    .into(),
                )),
            }
            .into(),
        )
    }

    fn test_3_substitutions(original: (i32, i32, i32), new: (i32, i32, i32)) {
        let (o0, o1, o2) = original;
        let (n0, n1, n2) = new;
        let expr = plus_times(o0, o1, o2);
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
            // As in `test_single_substitution`. Positions [1, 2, 0] take n0, n1 and n2,
            // so the constants become [n2, n0, n1].
            let expected = ErgoTree::new(ErgoTreeHeader::v0(true), &plus_times(n2, n0, n1))
                .unwrap()
                .sigma_serialize_bytes()
                .unwrap();
            assert_eq!(b.as_vec_u8(), expected);
        } else {
            unreachable!();
        }
    }
}
