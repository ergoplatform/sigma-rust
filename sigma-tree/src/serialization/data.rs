use crate::{
    ast::ConstantVal,
    ast::ConstantVal::*,
    ast::{CollPrim, ConstantColl},
    sigma_protocol,
    types::SType,
    types::SType::*,
};
use sigma_protocol::{SigmaBoolean, SigmaProp};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
};
use std::io;

pub struct DataSerializer {}

impl DataSerializer {
    pub fn sigma_serialize<W: WriteSigmaVlqExt>(
        c: &ConstantVal,
        w: &mut W,
    ) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L26-L26
        match c {
            Boolean(v) => w.put_u8(if *v { 1 } else { 0 }),
            Byte(v) => w.put_i8(*v),
            Short(v) => w.put_i16(*v),
            Int(v) => w.put_i32(*v),
            Long(v) => w.put_i64(*v),
            BigInt => todo!(),
            GroupElement => todo!(),
            SigmaProp(s) => s.value().sigma_serialize(w),
            CBox(_) => todo!(),
            AvlTree => todo!(),
            Coll(ct) => match ct {
                ConstantColl::Primitive(CollPrim::CollByte(b)) => {
                    w.put_usize_as_u16(b.len())?;
                    let ba: Vec<u8> = b.iter().map(|v| *v as u8).collect();
                    w.write_all(&ba[..])
                }
                ConstantColl::NonPrimitive { elem_tpe: _, v } => {
                    w.put_usize_as_u16(v.len())?;
                    v.iter()
                        .try_for_each(|e| DataSerializer::sigma_serialize(e, w))
                }
            },
            Tup(_) => todo!(),
        }
    }

    pub fn sigma_parse<R: ReadSigmaVlqExt>(
        tpe: &SType,
        r: &mut R,
    ) -> Result<ConstantVal, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        Ok(match tpe {
            SBoolean => Boolean(r.get_u8()? != 0),
            SByte => Byte(r.get_i8()?),
            SShort => Short(r.get_i16()?),
            SInt => Int(r.get_i32()?),
            SLong => Long(r.get_i64()?),
            SSigmaProp => ConstantVal::sigma_prop(SigmaProp::new(SigmaBoolean::sigma_parse(r)?)),
            SColl(elem_type) if **elem_type == SByte => {
                let len = r.get_u16()? as usize;
                let mut buf = vec![0u8; len];
                r.read_exact(&mut buf)?;
                Coll(ConstantColl::Primitive(CollPrim::CollByte(
                    buf.into_iter().map(|v| v as i8).collect(),
                )))
            }
            SColl(elem_type) => {
                let len = r.get_u16()? as usize;
                let mut elems = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    elems.push(DataSerializer::sigma_parse(elem_type, r)?);
                }
                Coll(ConstantColl::NonPrimitive {
                    elem_tpe: *elem_type.clone(),
                    v: elems,
                })
            }
            STup(types) => {
                let mut items = Vec::new();
                types.iter().try_for_each(|tpe| {
                    DataSerializer::sigma_parse(tpe, r).map(|v| items.push(v))
                })?;
                Tup(items)
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
