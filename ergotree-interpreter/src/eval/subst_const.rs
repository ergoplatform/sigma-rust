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
                                // fixme
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
        mir::{constant::Literal, expr::Expr},
        serialization::SigmaSerializable,
    };

    use crate::eval::tests::try_eval_out_wo_ctx;

    use super::*;
    #[test]
    fn test_get_constant() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(false),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr).unwrap();
        assert_eq!(ergo_tree.constants_len().unwrap(), 1);
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), false.into());

        let script_bytes =
            Expr::Const(Constant::from(ergo_tree.sigma_serialize_bytes().unwrap())).into();
        let positions = Expr::Const(Constant::from(vec![0])).into();
        let new_values = Expr::Const(Constant::from(vec![true])).into();
        let subst_const = Expr::SubstConstants(SubstConstants {
            script_bytes,
            positions,
            new_values,
        });
        let x: Value = try_eval_out_wo_ctx(&subst_const).unwrap();
        if let Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b))) = x {
            let new_ergo_tree = ErgoTree::sigma_parse_bytes(&b.as_vec_u8()).unwrap();
            assert_eq!(new_ergo_tree.constants_len().unwrap(), 1);
            assert_eq!(new_ergo_tree.get_constant(0).unwrap().unwrap(), true.into());
        } else {
            unreachable!();
        }
    }
}
