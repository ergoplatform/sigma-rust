use crate::{ast::CollPrim, ast::ConstantVal, ast::ConstantVal::*, types::SType, types::SType::*};
use sigma_ser::{
    serializer::SerializationError,
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
};
use std::io;

pub struct DataSerializer {}

impl DataSerializer {
    pub fn sigma_serialize<W: WriteSigmaVlqExt>(
        c: &ConstantVal,
        mut w: W,
    ) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L26-L26
        match c {
            Boolean(_) => todo!(),
            Byte(b) => w.put_i8(*b),
            Short(_) => todo!(),
            Int(_) => todo!(),
            Long(_) => todo!(),
            BigInt => todo!(),
            GroupElement => todo!(),
            SigmaProp(_) => todo!(),
            CBox(_) => todo!(),
            AvlTree => todo!(),
            ConstantVal::CollPrim(_) => todo!(),
            Coll(_) => todo!(),
            Tup(_) => todo!(),
        }
    }

    pub fn sigma_parse<R: ReadSigmaVlqExt>(
        tpe: &SType,
        mut r: R,
    ) -> Result<ConstantVal, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        let c = match tpe {
            SAny => todo!(),
            SByte => Byte(r.get_i8()?),
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
                    DataSerializer::sigma_parse(tpe, &mut r).map(|v| items.push(v))
                })?;
                Tup(items)
            }

            _ => todo!("handle the rest of the constant types"),
        };
        Ok(c)
    }
}
