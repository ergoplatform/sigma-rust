use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::ergo_tree::substitute_constants;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::subst_const::SubstConstants;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;
use ergotree_ir::util::AsVecI8;
use ergotree_ir::util::AsVecU8;
use std::convert::TryFrom;

impl Evaluable for SubstConstants {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let script_bytes_v = self.script_bytes.eval(env, ctx)?;
        let positions_v = self.positions.eval(env, ctx)?;
        let new_values_v = self.new_values.eval(env, ctx)?;

        let positions: Vec<usize> =
            if let Value::Coll(CollKind::WrappedColl { elem_tpe, items }) = positions_v {
                if elem_tpe != SType::SInt {
                    return Err(EvalError::Misc(String::from(
                        "SubstConstants: expected evaluation of `positions` be of type `Coll[SInt]`\
                        , got Coll[_] instead",
                    )));
                } else {
                    items
                        .into_iter()
                        .map(|v| {
                            if let Value::Int(i) = v {
                                i as usize
                            } else {
                                // Can't reach here since sigma-parsing ensures each item is of the
                                // same type.
                                unreachable!();
                            }
                        })
                        .collect()
                }
            } else {
                return Err(EvalError::Misc(String::from(
                    "SubstConstants: expected evaluation of `positions` be of type `Coll[SInt]`\
                 , got _ instead",
                )));
            };

        let (new_constants_type, new_constants) =
            if let Value::Coll(CollKind::WrappedColl { elem_tpe, items }) = new_values_v {
                let mut items_const = vec![];
                for v in items {
                    let c = Constant::try_from(v).map_err(EvalError::Misc)?;
                    items_const.push(c);
                }
                (elem_tpe, items_const)
            } else {
                return Err(EvalError::Misc(String::from(
                    "SubstConstants: expected evaluation of `new_values` be of type `Coll[_]`, got \
                    _ instead",
                )));
            };

        if let Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b))) = script_bytes_v {
            let data =
                substitute_constants(b.as_vec_u8(), positions, new_constants, new_constants_type)
                    .map_err(|e| EvalError::Misc(format!("{:?}", e)))?;
            Ok(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(
                data.as_vec_i8(),
            ))))
        } else {
            Err(EvalError::Misc(String::from(
                "SubstConstants: expected evaluation of \
                 `script_bytes` be of type `Coll[SBytes]`, got _ instead",
            )))
        }
    }
}

#[cfg(test)]
//#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[allow(clippy::expect_used)]
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

    use crate::eval::tests::try_eval_out_wo_ctx;

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

        let subst_const = Expr::SubstConstants(SubstConstants {
            script_bytes,
            positions,
            new_values,
        });

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
        let expr = Expr::BinOp(BinOp {
            kind: BinOpKind::Arith(ArithOp::Plus),
            left: Box::new(Expr::Const(o0.into())),
            right: Box::new(Expr::BinOp(BinOp {
                kind: BinOpKind::Arith(ArithOp::Multiply),
                left: Box::new(Expr::Const(o1.into())),
                right: Box::new(Expr::Const(o2.into())),
            })),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr).unwrap();
        assert_eq!(ergo_tree.constants_len().unwrap(), 3);
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), o0.into());
        assert_eq!(ergo_tree.get_constant(1).unwrap().unwrap(), o1.into());
        assert_eq!(ergo_tree.get_constant(2).unwrap().unwrap(), o2.into());

        let script_bytes =
            Expr::Const(Constant::from(ergo_tree.sigma_serialize_bytes().unwrap())).into();

        let positions = Expr::Const(Constant::from(vec![1, 2, 0])).into();

        let new_values = Expr::Const(Constant::from(vec![n0, n1, n2])).into();

        let subst_const = Expr::SubstConstants(SubstConstants {
            script_bytes,
            positions,
            new_values,
        });

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
}
