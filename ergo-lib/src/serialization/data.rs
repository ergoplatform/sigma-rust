use crate::ast::constant::TryExtractFromError;
use crate::ast::constant::TryExtractInto;
use crate::ast::value::CollKind;
use crate::ast::value::CollPrim;
use crate::ast::value::Value;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use crate::sigma_protocol::{
    dlog_group::EcPoint, sigma_boolean::SigmaBoolean, sigma_boolean::SigmaProp,
};
use crate::types::stuple;
use crate::types::stype::SType;
use crate::util::AsVecU8;

use super::sigma_byte_writer::SigmaByteWrite;
use std::convert::TryInto;
use std::io;

pub struct DataSerializer {}

impl DataSerializer {
    pub fn sigma_serialize<W: SigmaByteWrite>(c: &Value, w: &mut W) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L26-L26
        match c {
            Value::Boolean(v) => w.put_u8(if *v { 1 } else { 0 }),
            Value::Byte(v) => w.put_i8(*v),
            Value::Short(v) => w.put_i16(*v),
            Value::Int(v) => w.put_i32(*v),
            Value::Long(v) => w.put_i64(*v),
            Value::BigInt => todo!(),
            Value::GroupElement(ecp) => ecp.sigma_serialize(w),
            Value::SigmaProp(s) => s.value().sigma_serialize(w),
            Value::CBox(_) => todo!(),
            Value::AvlTree => todo!(),
            Value::Coll(ct) => match ct {
                CollKind::Primitive(CollPrim::CollByte(b)) => {
                    w.put_usize_as_u16(b.len())?;
                    w.write_all(b.clone().as_vec_u8().as_slice())
                }
                CollKind::NonPrimitive {
                    elem_tpe: SType::SBoolean,
                    v,
                } => {
                    w.put_usize_as_u16(v.len())?;
                    let maybe_bools: Result<Vec<bool>, TryExtractFromError> = v
                        .clone()
                        .into_iter()
                        .map(|i| i.try_extract_into::<bool>())
                        .collect();
                    w.put_bits(maybe_bools.unwrap().as_slice())
                }
                CollKind::NonPrimitive { elem_tpe: _, v } => {
                    w.put_usize_as_u16(v.len())?;
                    v.iter()
                        .try_for_each(|e| DataSerializer::sigma_serialize(e, w))
                }
            },
            Value::Tup(items) => items
                .iter()
                .try_for_each(|i| DataSerializer::sigma_serialize(i, w)),
            Value::Opt(_) => panic!("Option is not yet supported"), // unsupported, see https://github.com/ScorexFoundation/sigmastate-interpreter/issues/659
            _ => panic!("serialization is not supported for value: {0:?}", c),
        }
    }

    pub fn sigma_parse<R: SigmaByteRead>(
        tpe: &SType,
        r: &mut R,
    ) -> Result<Value, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        use SType::*;
        Ok(match tpe {
            SBoolean => Value::Boolean(r.get_u8()? != 0),
            SByte => Value::Byte(r.get_i8()?),
            SShort => Value::Short(r.get_i16()?),
            SInt => Value::Int(r.get_i32()?),
            SLong => Value::Long(r.get_i64()?),
            SGroupElement => Value::GroupElement(Box::new(EcPoint::sigma_parse(r)?)),
            SSigmaProp => Value::sigma_prop(SigmaProp::new(SigmaBoolean::sigma_parse(r)?)),
            SColl(elem_type) if **elem_type == SByte => {
                let len = r.get_u16()? as usize;
                let mut buf = vec![0u8; len];
                r.read_exact(&mut buf)?;
                Value::Coll(CollKind::Primitive(CollPrim::CollByte(
                    buf.into_iter().map(|v| v as i8).collect(),
                )))
            }
            SColl(elem_type) if **elem_type == SBoolean => {
                let len = r.get_u16()? as usize;
                let bools = r.get_bits(len)?;
                Value::Coll(CollKind::NonPrimitive {
                    elem_tpe: *elem_type.clone(),
                    v: bools.into_iter().map(|b| b.into()).collect(),
                })
            }
            SColl(elem_type) => {
                let len = r.get_u16()? as usize;
                let mut elems = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    elems.push(DataSerializer::sigma_parse(elem_type, r)?);
                }
                Value::Coll(CollKind::NonPrimitive {
                    elem_tpe: *elem_type.clone(),
                    v: elems,
                })
            }
            STuple(stuple::STuple { items: types }) => {
                let mut items = Vec::new();
                types.iter().try_for_each(|tpe| {
                    DataSerializer::sigma_parse(tpe, r).map(|v| items.push(v))
                })?;
                // we get the tuple item value for each tuple item type,
                // since items types quantity has checked bounds, we can be sure that items count
                // is correct
                Value::Tup(items.try_into().unwrap())
            }

            c => {
                return Err(SerializationError::NotImplementedYet(format!(
                    "parsing of constant value of type {:?} is not yet supported",
                    c
                )))
            }
        })
    }
}
