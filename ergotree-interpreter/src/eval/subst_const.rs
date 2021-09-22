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
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteReader;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::types::stype::SType;
use ergotree_ir::util::AsVecU8;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use std::io::Cursor;

impl Evaluable for SubstConstants {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let script_bytes_v = self.script_bytes.eval(env, ctx)?;
        let positions_v = self.positions.eval(env, ctx)?;
        let new_values_v = self.new_values.eval(env, ctx)?;

        let positions: Vec<usize> =
            if let Value::Coll(CollKind::WrappedColl { elem_tpe, items }) = positions_v {
                if elem_tpe != SType::SInt {
                    return Err(EvalError::InvalidResultType);
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
                // fixme
                return Err(EvalError::InvalidResultType);
            };

        let (new_values_type, new_values) =
            if let Value::Coll(CollKind::WrappedColl { elem_tpe, items }) = new_values_v {
                (elem_tpe, items)
            } else {
                // fixme
                return Err(EvalError::InvalidResultType);
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
                    for e in res.iter_mut() {
                        *e = Constant::sigma_parse(&mut r).map_err(EvalError::SigmaParsingError)?;
                    }
                    Some(res)
                } else {
                    None
                }
            } else {
                None
            };
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
                    } else {
                        // No substitution
                        c.sigma_serialize(&mut w)
                            .map_err(EvalError::SigmaSerializationError)?;
                    }
                }
            }
        }
        Err(EvalError::InvalidResultType)
    }
}

fn map_parsing_err(io_err: std::io::Error) -> EvalError {
    EvalError::SigmaParsingError(SigmaParsingError::Io(format!("{}", io_err)))
}

fn map_serialization_err<T: std::fmt::Debug>(e: T) -> EvalError {
    EvalError::SigmaSerializationError(SigmaSerializationError::Io(format!("{:?}", e)))
}
