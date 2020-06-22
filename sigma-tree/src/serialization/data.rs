use crate::{
    ast::CollPrim, ast::ConstantVal, ast::ConstantVal::*, sigma_protocol, types::SType,
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
            ConstantVal::CollPrim(c) => match c {
                CollPrim::CollByte(b) => {
                    w.put_usize_as_u16(b.len())?;
                    let ba: Vec<u8> = b.iter().map(|v| *v as u8).collect();
                    w.write_all(&ba[..])
                }
                CollPrim::CollBoolean(_) => todo!(),
                CollPrim::CollShort(_) => todo!(),
                CollPrim::CollInt(_) => todo!(),
                CollPrim::CollLong(_) => todo!(),
            },
            Coll(_) => todo!(),
            Tup(_) => todo!(),
        }
    }

    pub fn sigma_parse<R: ReadSigmaVlqExt>(
        tpe: &SType,
        r: &mut R,
    ) -> Result<ConstantVal, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        let c = match tpe {
            SAny => todo!(),
            SBoolean => Boolean(r.get_u8()? != 0),
            SByte => Byte(r.get_i8()?),
            SShort => Short(r.get_i16()?),
            SInt => Int(r.get_i32()?),
            SLong => Long(r.get_i64()?),
            SSigmaProp => ConstantVal::sigma_prop(SigmaProp::new(SigmaBoolean::sigma_parse(r)?)),
            SColl(elem_type) => {
                let len = r.get_u16()? as usize;
                if **elem_type == SByte {
                    let mut buf = vec![0u8; len];
                    r.read_exact(&mut buf)?;
                    CollPrim(CollPrim::CollByte(
                        buf.into_iter().map(|v| v as i8).collect(),
                    ))
                } else {
                    todo!("handle the rest of supported collection types");
                }
            }
            STup(types) => {
                let mut items = Vec::new();
                types.iter().try_for_each(|tpe| {
                    DataSerializer::sigma_parse(tpe, r).map(|v| items.push(v))
                })?;
                Tup(items)
            }

            _ => todo!("handle the rest of the constant types"),
        };
        Ok(c)
    }
}
