use crate::{
    data::CCollPrim, data::ConstantKind, data::ConstantKind::*, types::SType, types::SType::*,
};
use sigma_ser::{
    serializer::SerializationError,
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
};
use std::io;

// TODO: extract
pub struct DataSerializer {}

impl DataSerializer {
    pub fn sigma_serialize<W: WriteSigmaVlqExt>(
        c: &ConstantKind,
        tpe: &SType,
        w: W,
    ) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L26-L26
        todo!()
    }

    pub fn sigma_parse<R: ReadSigmaVlqExt>(
        tpe: &SType,
        mut r: R,
    ) -> Result<ConstantKind, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        let c = match tpe {
            SAny => todo!(),
            SByte => CByte(r.get_i8()?),
            SColl(elem_type) => {
                let len = r.get_u16()? as usize;
                if **elem_type == SByte {
                    let mut buf = vec![0u8; len];
                    r.read_exact(&mut buf)?;
                    CCollPrim(CCollPrim::CCollByte(
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
                CTup(items)
            }

            _ => todo!("handle the rest of the constant types"),
        };
        Ok(c)
    }
}
