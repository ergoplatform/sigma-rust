use crate::mir::constant::Literal;
use crate::mir::constant::TryExtractFromError;
use crate::mir::constant::TryExtractInto;
use crate::mir::value::NativeColl;
use crate::serialization::SigmaSerializationError;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::sigma_protocol::{
    dlog_group::EcPoint, sigma_boolean::SigmaBoolean, sigma_boolean::SigmaProp,
};
use crate::types::stuple;
use crate::types::stype::SType;
use crate::util::AsVecU8;

use super::sigma_byte_writer::SigmaByteWrite;
use std::convert::TryInto;

pub struct DataSerializer {}

impl DataSerializer {
    //pub fn sigma_serialize<W: SigmaByteWrite>(c: &Value, w: &mut W) -> SigmaSerializeResult {
    //    // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L26-L26
    //    Ok(match c {
    //        Value::Boolean(v) => w.put_u8(if *v { 1 } else { 0 })?,
    //        Value::Byte(v) => w.put_i8(*v)?,
    //        Value::Short(v) => w.put_i16(*v)?,
    //        Value::Int(v) => w.put_i32(*v)?,
    //        Value::Long(v) => w.put_i64(*v)?,
    //        Value::BigInt(v) => {
    //            let bytes = v.to_signed_bytes_be();
    //            w.put_u16(bytes.len() as u16)?;
    //            w.write_all(&bytes)?
    //        }
    //        Value::GroupElement(ecp) => ecp.sigma_serialize(w)?,
    //        Value::SigmaProp(s) => s.value().sigma_serialize(w)?,
    //        Value::CBox(_) => return Err(SigmaSerializationError::NotImplementedYet("Box")),
    //        Value::AvlTree => return Err(SigmaSerializationError::NotImplementedYet("AvlTree")),
    //        Value::Coll(ct) => match ct {
    //            CollKind::NativeColl(NativeColl::CollByte(b)) => {
    //                w.put_usize_as_u16_unwrapped(b.len())?;
    //                w.write_all(b.clone().as_vec_u8().as_slice())?
    //            }
    //            CollKind::WrappedColl {
    //                elem_tpe: SType::SBoolean,
    //                items: v,
    //            } => {
    //                w.put_usize_as_u16_unwrapped(v.len())?;
    //                let maybe_bools: Result<Vec<bool>, TryExtractFromError> = v
    //                    .clone()
    //                    .into_iter()
    //                    .map(|i| i.try_extract_into::<bool>())
    //                    .collect();
    //                w.put_bits(maybe_bools?.as_slice())?
    //            }
    //            CollKind::WrappedColl {
    //                elem_tpe: _,
    //                items: v,
    //            } => {
    //                w.put_usize_as_u16_unwrapped(v.len())?;
    //                v.iter()
    //                    .try_for_each(|e| DataSerializer::sigma_serialize(e, w))?
    //            }
    //        },
    //        Value::Tup(items) => items
    //            .iter()
    //            .try_for_each(|i| DataSerializer::sigma_serialize(i, w))?,
    //        // unsupported, see
    //        // https://github.com/ScorexFoundation/sigmastate-interpreter/issues/659
    //        Value::Opt(_) => {
    //            return Err(SigmaSerializationError::NotSupported("Option"));
    //        }
    //        Value::Context => return Err(SigmaSerializationError::NotSupported("Context")),
    //        Value::Global => return Err(SigmaSerializationError::NotSupported("Global")),
    //        Value::Lambda(_) => return Err(SigmaSerializationError::NotSupported("Lambda")),
    //    })
    //}

    pub fn sigma_serialize_literal<W: SigmaByteWrite>(
        c: &Literal,
        w: &mut W,
    ) -> SigmaSerializeResult {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L26-L26
        Ok(match c {
            Literal::Boolean(v) => w.put_u8(if *v { 1 } else { 0 })?,
            Literal::Byte(v) => w.put_i8(*v)?,
            Literal::Short(v) => w.put_i16(*v)?,
            Literal::Int(v) => w.put_i32(*v)?,
            Literal::Long(v) => w.put_i64(*v)?,
            Literal::BigInt(v) => {
                let bytes = v.to_signed_bytes_be();
                w.put_u16(bytes.len() as u16)?;
                w.write_all(&bytes)?
            }
            Literal::GroupElement(ecp) => ecp.sigma_serialize(w)?,
            Literal::SigmaProp(s) => s.value().sigma_serialize(w)?,
            Literal::CBox(_) => return Err(SigmaSerializationError::NotImplementedYet("Box")),
            Literal::Coll(ct) => match ct {
                crate::mir::constant::CollKind::NativeColl(NativeColl::CollByte(b)) => {
                    w.put_usize_as_u16_unwrapped(b.len())?;
                    w.write_all(b.clone().as_vec_u8().as_slice())?
                }
                crate::mir::constant::CollKind::WrappedColl {
                    elem_tpe: SType::SBoolean,
                    items: v,
                } => {
                    w.put_usize_as_u16_unwrapped(v.len())?;
                    let maybe_bools: Result<Vec<bool>, TryExtractFromError> = v
                        .clone()
                        .into_iter()
                        .map(|i| i.try_extract_into::<bool>())
                        .collect();
                    w.put_bits(maybe_bools?.as_slice())?
                }
                crate::mir::constant::CollKind::WrappedColl {
                    elem_tpe: _,
                    items: v,
                } => {
                    w.put_usize_as_u16_unwrapped(v.len())?;
                    v.iter()
                        .try_for_each(|e| DataSerializer::sigma_serialize_literal(e, w))?
                }
            },
            Literal::Tup(items) => items
                .iter()
                .try_for_each(|i| DataSerializer::sigma_serialize_literal(i, w))?,
            // unsupported, see
            // https://github.com/ScorexFoundation/sigmastate-interpreter/issues/659
            Literal::Opt(_) => {
                return Err(SigmaSerializationError::NotSupported("Option"));
            }
        })
    }
    //pub fn sigma_parse<R: SigmaByteRead>(
    //    tpe: &SType,
    //    r: &mut R,
    //) -> Result<Value, SigmaParsingError> {
    //    // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
    //    use SType::*;
    //    Ok(match tpe {
    //        SBoolean => Value::Boolean(r.get_u8()? != 0),
    //        SByte => Value::Byte(r.get_i8()?),
    //        SShort => Value::Short(r.get_i16()?),
    //        SInt => Value::Int(r.get_i32()?),
    //        SLong => Value::Long(r.get_i64()?),
    //        SBigInt => {
    //            let size = r.get_u16()?;
    //            if size > 32 {
    //                return Err(SigmaParsingError::ValueOutOfBounds(format!(
    //                    "serialized BigInt size {0} bytes exceeds 32",
    //                    size
    //                )));
    //            }
    //            let mut buf = vec![0u8; size as usize];
    //            r.read_exact(&mut buf)?;
    //            match buf.as_slice().try_into() {
    //                Ok(x) => Value::BigInt(x),
    //                Err(e) => return Err(SigmaParsingError::ValueOutOfBounds(e)),
    //            }
    //        }
    //        SGroupElement => Value::GroupElement(Box::new(EcPoint::sigma_parse(r)?)),
    //        SSigmaProp => Value::sigma_prop(SigmaProp::new(SigmaBoolean::sigma_parse(r)?)),
    //        SColl(elem_type) if **elem_type == SByte => {
    //            let len = r.get_u16()? as usize;
    //            let mut buf = vec![0u8; len];
    //            r.read_exact(&mut buf)?;
    //            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(
    //                buf.into_iter().map(|v| v as i8).collect(),
    //            )))
    //        }
    //        SColl(elem_type) if **elem_type == SBoolean => {
    //            let len = r.get_u16()? as usize;
    //            let bools = r.get_bits(len)?;
    //            Value::Coll(CollKind::WrappedColl {
    //                elem_tpe: *elem_type.clone(),
    //                items: bools.into_iter().map(|b| b.into()).collect(),
    //            })
    //        }
    //        SColl(elem_type) => {
    //            let len = r.get_u16()? as usize;
    //            let mut elems = Vec::with_capacity(len as usize);
    //            for _ in 0..len {
    //                elems.push(DataSerializer::sigma_parse(elem_type, r)?);
    //            }
    //            Value::Coll(CollKind::WrappedColl {
    //                elem_tpe: *elem_type.clone(),
    //                items: elems,
    //            })
    //        }
    //        STuple(stuple::STuple { items: types }) => {
    //            let mut items = Vec::new();
    //            types.iter().try_for_each(|tpe| {
    //                DataSerializer::sigma_parse(tpe, r).map(|v| items.push(v))
    //            })?;
    //            // we get the tuple item value for each tuple item type,
    //            // since items types quantity has checked bounds, we can be sure that items count
    //            // is correct
    //            Value::Tup(items.try_into()?)
    //        }
    //        SBox => {
    //            return Err(SigmaParsingError::NotImplementedYet(
    //                "SBox data".to_string(),
    //            ))
    //        }
    //        SAvlTree => {
    //            return Err(SigmaParsingError::NotImplementedYet(
    //                "SAvlTree data".to_string(),
    //            ))
    //        }
    //        STypeVar(_) => return Err(SigmaParsingError::NotSupported("TypeVar data")),
    //        SAny => return Err(SigmaParsingError::NotSupported("SAny data")),
    //        SOption(_) => return Err(SigmaParsingError::NotSupported("SOption data")),
    //        SFunc(_) => return Err(SigmaParsingError::NotSupported("SFunc data")),
    //        SContext => return Err(SigmaParsingError::NotSupported("SContext data")),
    //        SHeader => return Err(SigmaParsingError::NotSupported("SHeader data")),
    //        SPreHeader => return Err(SigmaParsingError::NotSupported("SPreHeader data")),
    //        SGlobal => return Err(SigmaParsingError::NotSupported("SGlobal data")),
    //    })
    //}
    pub fn sigma_parse_literal<R: SigmaByteRead>(
        tpe: &SType,
        r: &mut R,
    ) -> Result<Literal, SigmaParsingError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        use SType::*;
        Ok(match tpe {
            SBoolean => Literal::Boolean(r.get_u8()? != 0),
            SByte => Literal::Byte(r.get_i8()?),
            SShort => Literal::Short(r.get_i16()?),
            SInt => Literal::Int(r.get_i32()?),
            SLong => Literal::Long(r.get_i64()?),
            SBigInt => {
                let size = r.get_u16()?;
                if size > 32 {
                    return Err(SigmaParsingError::ValueOutOfBounds(format!(
                        "serialized BigInt size {0} bytes exceeds 32",
                        size
                    )));
                }
                let mut buf = vec![0u8; size as usize];
                r.read_exact(&mut buf)?;
                match buf.as_slice().try_into() {
                    Ok(x) => Literal::BigInt(x),
                    Err(e) => return Err(SigmaParsingError::ValueOutOfBounds(e)),
                }
            }
            SGroupElement => Literal::GroupElement(Box::new(EcPoint::sigma_parse(r)?)),
            SSigmaProp => {
                Literal::SigmaProp(Box::new(SigmaProp::new(SigmaBoolean::sigma_parse(r)?)))
            }
            SColl(elem_type) if **elem_type == SByte => {
                let len = r.get_u16()? as usize;
                let mut buf = vec![0u8; len];
                r.read_exact(&mut buf)?;
                Literal::Coll(crate::mir::constant::CollKind::NativeColl(
                    NativeColl::CollByte(buf.into_iter().map(|v| v as i8).collect()),
                ))
            }
            SColl(elem_type) if **elem_type == SBoolean => {
                let len = r.get_u16()? as usize;
                let bools = r.get_bits(len)?;
                Literal::Coll(crate::mir::constant::CollKind::WrappedColl {
                    elem_tpe: *elem_type.clone(),
                    items: bools.into_iter().map(|b| b.into()).collect(),
                })
            }
            SColl(elem_type) => {
                let len = r.get_u16()? as usize;
                let mut elems = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    elems.push(DataSerializer::sigma_parse_literal(elem_type, r)?);
                }
                Literal::Coll(crate::mir::constant::CollKind::WrappedColl {
                    elem_tpe: *elem_type.clone(),
                    items: elems,
                })
            }
            STuple(stuple::STuple { items: types }) => {
                let mut items = Vec::new();
                types.iter().try_for_each(|tpe| {
                    DataSerializer::sigma_parse_literal(tpe, r).map(|v| items.push(v))
                })?;
                // we get the tuple item value for each tuple item type,
                // since items types quantity has checked bounds, we can be sure that items count
                // is correct
                Literal::Tup(items.try_into()?)
            }
            SBox => {
                return Err(SigmaParsingError::NotImplementedYet(
                    "SBox data".to_string(),
                ))
            }
            SAvlTree => {
                return Err(SigmaParsingError::NotImplementedYet(
                    "SAvlTree data".to_string(),
                ))
            }
            STypeVar(_) => return Err(SigmaParsingError::NotSupported("TypeVar data")),
            SAny => return Err(SigmaParsingError::NotSupported("SAny data")),
            SOption(_) => return Err(SigmaParsingError::NotSupported("SOption data")),
            SFunc(_) => return Err(SigmaParsingError::NotSupported("SFunc data")),
            SContext => return Err(SigmaParsingError::NotSupported("SContext data")),
            SHeader => return Err(SigmaParsingError::NotSupported("SHeader data")),
            SPreHeader => return Err(SigmaParsingError::NotSupported("SPreHeader data")),
            SGlobal => return Err(SigmaParsingError::NotSupported("SGlobal data")),
        })
    }
}
