use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::ergo_tree::ErgoTreeHeader;
use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::subst_const::SubstConstants;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::constant_store::ConstantStore;
use ergotree_ir::serialization::data::DataSerializer;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteReader;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::types::stype::SType;
use ergotree_ir::util::AsVecI8;
use ergotree_ir::util::AsVecU8;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use std::io::Cursor;
use std::io::Read;

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

        let (new_values_type, new_values) =
            if let Value::Coll(CollKind::WrappedColl { elem_tpe, items }) = new_values_v {
                (elem_tpe, items)
            } else {
                return Err(EvalError::Misc(String::from(
                    "SubstConstants: expected evaluation of `new_values` be of type `Coll[_]`, got \
                    _ instead",
                )));
            };

        if let Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b))) = script_bytes_v {
            let mut data = Vec::new();
            let mut w = SigmaByteWriter::new(&mut data, None);
            let cursor = Cursor::new(b.as_vec_u8());
            let mut r = SigmaByteReader::new(cursor, ConstantStore::empty());

            // Check the header
            let header = ErgoTreeHeader::sigma_parse(&mut r).map_err(map_parsing_err)?;
            if header.version() != ErgoTreeVersion::V0 && !header.has_size() {
                return Err(EvalError::Misc(format!(
                    "Invalid ErgoTreeHeader, size bit is expected for version {:?}",
                    header.version()
                )));
            }
            if header.has_size() {
                r.get_u32().map_err(map_serialization_err)?;
            }

            // Deserialize the constants within the script
            let constants = if header.is_constant_segregation() {
                let num_consts = r.get_u32().map_err(map_serialization_err)?;

                if num_consts > 0 {
                    let mut res = Vec::with_capacity(num_consts as usize);
                    for _ in 0..num_consts {
                        res.push(
                            Constant::sigma_parse(&mut r).map_err(EvalError::SigmaParsingError)?,
                        );
                    }
                    Some(res)
                } else {
                    None
                }
            } else {
                None
            };

            // Now capture the remaining tree bytes
            let mut buf = vec![0_u8; b.len()];
            let num_tree_bytes = r
                .read_to_end(&mut buf)
                .map_err(|e| EvalError::Misc(format!("{}", e)))?;
            let constants_len = if let Some(c) = constants.as_ref() {
                c.len() as u32
            } else {
                0
            };
            header
                .sigma_serialize(&mut w)
                .map_err(map_serialization_err)?;

            // TODO from sigmastate: don't serialize the following when segregation is off
            w.put_u32(constants_len).map_err(map_serialization_err)?;

            // Substitute constants
            if let Some(constants) = constants {
                for (i, c) in constants.iter().enumerate() {
                    if let Some(ix) = positions.iter().position(|j| *j == i) {
                        if c.tpe == new_values[ix].tpe() && c.tpe == new_values_type {
                            c.tpe.sigma_serialize(&mut w)?;
                            DataSerializer::sigma_serialize_value(&new_values[ix], &mut w)?;
                        }
                    } else {
                        // No substitution
                        c.sigma_serialize(&mut w)
                            .map_err(EvalError::SigmaSerializationError)?;
                    }
                }
            }

            // Extend with unmodified tree bytes.
            data.extend_from_slice(&buf[0..num_tree_bytes]);
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

fn map_parsing_err(io_err: std::io::Error) -> EvalError {
    EvalError::SigmaParsingError(SigmaParsingError::Io(format!("{}", io_err)))
}

fn map_serialization_err<T: std::fmt::Debug>(e: T) -> EvalError {
    EvalError::SigmaSerializationError(SigmaSerializationError::Io(format!("{:?}", e)))
}

#[cfg(test)]
//#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[allow(clippy::expect_used)]
mod tests {
    use ergotree_ir::{
        ergo_tree::ErgoTree,
        mir::{constant::Literal, expr::Expr},
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
