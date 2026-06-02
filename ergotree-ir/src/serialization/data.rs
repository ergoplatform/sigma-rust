use alloc::boxed::Box;
use ergo_chain_types::Header;
use sigma_ser::ScorexSerializable;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use sigma_util::AsVecU8;

use crate::bigint256::BigInt256;
use crate::chain::ergo_box::ErgoBox;
use crate::ergo_tree::ErgoTreeVersion;
use crate::mir::avl_tree_data::AvlTreeData;
use crate::mir::constant::Literal;
use crate::mir::constant::TryExtractFromError;
use crate::mir::constant::TryExtractInto;
use crate::mir::value::CollKind;
use crate::mir::value::NativeColl;
use crate::serialization::SigmaSerializationError;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::sigma_protocol::{sigma_boolean::SigmaBoolean, sigma_boolean::SigmaProp};
use crate::types::stuple;
use crate::types::stype::SType;
use crate::unsignedbigint256::UnsignedBigInt;
use ergo_chain_types::EcPoint;

use super::sigma_byte_writer::SigmaByteWrite;
use alloc::sync::Arc;
use core::convert::TryInto;

/// Used to serialize and parse `Literal` and `Value`.
pub struct DataSerializer {}

impl DataSerializer {
    /// Serialize Literal without typecode serialized
    pub fn sigma_serialize<W: SigmaByteWrite>(c: &Literal, w: &mut W) -> SigmaSerializeResult {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L26-L26
        Ok(match c {
            Literal::Unit => (),
            Literal::Boolean(v) => {
                w.put_u8(u8::from(*v))?;
                w.add_put_byte_cost();
            }
            Literal::Byte(v) => {
                w.put_i8(*v)?;
                w.add_put_byte_cost();
            }
            Literal::Short(v) => {
                w.put_i16(*v)?;
                w.add_put_numeric_cost();
            }
            Literal::Int(v) => {
                w.put_i32(*v)?;
                w.add_put_numeric_cost();
            }
            Literal::Long(v) => {
                w.put_i64(*v)?;
                w.add_put_numeric_cost();
            }
            Literal::BigInt(v) => {
                v.sigma_serialize(w)?;
            }
            Literal::String(s) => {
                w.put_usize_as_u32_unwrapped(s.len())?;
                w.add_put_numeric_cost();
                w.write_all(s.as_bytes())?;
                w.add_put_chunk_cost(s.len());
            }
            Literal::GroupElement(ecp) => {
                // EcPoint::scorex_serialize writes exactly GROUP_SIZE (33) bytes as one block;
                // Scala meters it as putBytes(33) = PutChunkCost. (EcPoint is in ergo-chain-types
                // and can't reach the cost sink, so record it here at the delegating site.)
                ecp.sigma_serialize(w)?;
                w.add_put_chunk_cost(EcPoint::GROUP_SIZE);
            }
            Literal::SigmaProp(s) => s.value().sigma_serialize(w)?,
            Literal::UnsignedBigInt(v) if w.tree_version() >= ErgoTreeVersion::V3 => {
                v.sigma_serialize(w)?
            }
            Literal::UnsignedBigInt(_) => {
                return Err(SigmaSerializationError::NotSupported(
                    "Can't serialize UnsignedBigInt with tree version < 3".into(),
                ))
            }
            Literal::AvlTree(a) => a.sigma_serialize(w)?,
            Literal::CBox(b) => b.sigma_serialize(w)?,
            Literal::Coll(ct) => match ct {
                CollKind::NativeColl(NativeColl::CollByte(b)) => {
                    w.put_usize_as_u16_unwrapped(b.len())?;
                    w.add_put_numeric_cost();
                    w.write_all(b.clone().as_vec_u8().as_slice())?;
                    w.add_put_chunk_cost(b.len());
                }
                CollKind::WrappedColl {
                    elem_tpe: SType::SBoolean,
                    items: v,
                } => {
                    w.put_usize_as_u16_unwrapped(v.len())?;
                    w.add_put_numeric_cost();
                    let maybe_bools: Result<Vec<bool>, TryExtractFromError> = v
                        .clone()
                        .iter()
                        .cloned()
                        .map(|i| i.try_extract_into::<bool>())
                        .collect();
                    w.put_bits(maybe_bools?.as_slice())?;
                    w.add_put_chunk_cost(v.len());
                }
                CollKind::WrappedColl {
                    elem_tpe: _,
                    items: v,
                } => {
                    w.put_usize_as_u16_unwrapped(v.len())?;
                    w.add_put_numeric_cost();
                    v.iter()
                        .try_for_each(|e| DataSerializer::sigma_serialize(e, w))?
                }
            },
            Literal::Tup(items) => items
                .iter()
                .try_for_each(|i| DataSerializer::sigma_serialize(i, w))?,
            Literal::Header(h) if w.tree_version() >= ErgoTreeVersion::V3 => {
                h.scorex_serialize(w)?;
            }
            Literal::Opt(opt) if w.tree_version() >= ErgoTreeVersion::V3 => {
                w.put_option(Option::as_ref(opt), |w, v| {
                    DataSerializer::sigma_serialize(v, w)
                })?;
                w.add_put_byte_cost();
            }
            // unsupported, see
            // https://github.com/ScorexFoundation/sigmastate-interpreter/issues/659
            Literal::Opt(_) => {
                return Err(SigmaSerializationError::NotSupported(
                    "Option serialization is not supported".to_string(),
                ));
            }
            Literal::Header(_) => {
                return Err(SigmaSerializationError::NotSupported(
                    "Header serialization is not supported".to_string(),
                ));
            }
        })
    }

    /// Parse sigma-serialized literal
    pub fn sigma_parse<R: SigmaByteRead>(
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
            SString => {
                let len = r.get_u32()?;
                let mut buf = vec![0; len as usize];
                r.read_exact(&mut buf)?;
                Literal::String(String::from_utf8_lossy(&buf).into())
            }
            SBigInt => Literal::BigInt(BigInt256::sigma_parse(r)?),
            SUnit => Literal::Unit,
            SGroupElement => Literal::GroupElement(Arc::new(EcPoint::sigma_parse(r)?)),
            SSigmaProp => {
                Literal::SigmaProp(Box::new(SigmaProp::new(SigmaBoolean::sigma_parse(r)?)))
            }
            SUnsignedBigInt if r.tree_version() >= ErgoTreeVersion::V3 => {
                Literal::UnsignedBigInt(UnsignedBigInt::sigma_parse(r)?)
            }
            SColl(elem_type) if **elem_type == SByte => {
                let len = r.get_u16()? as usize;
                let mut buf = vec![0u8; len];
                r.read_exact(&mut buf)?;
                Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(
                    buf.into_iter().map(|v| v as i8).collect(),
                )))
            }
            SColl(elem_type) if **elem_type == SBoolean => {
                let len = r.get_u16()? as usize;
                let bools = r.get_bits(len)?;
                Literal::Coll(CollKind::WrappedColl {
                    elem_tpe: (**elem_type).clone(),
                    items: bools.into_iter().map(|b| b.into()).collect(),
                })
            }
            SColl(elem_type) => {
                let len = r.get_u16()? as usize;
                let elems = (0..len)
                    .map(|_| DataSerializer::sigma_parse(elem_type, r))
                    .collect::<Result<Arc<[_]>, SigmaParsingError>>()?;
                Literal::Coll(CollKind::WrappedColl {
                    elem_tpe: (**elem_type).clone(),
                    items: elems,
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
                Literal::Tup(items.try_into()?)
            }
            SUnsignedBigInt => {
                return Err(SigmaParsingError::NotSupported(
                    "UnsignedBigInt can't be serialized on tree versions < 3",
                ))
            }
            SOption(inner_tpe) if r.tree_version() >= ErgoTreeVersion::V3 => {
                let res = r.get_option(|r| DataSerializer::sigma_parse(inner_tpe, r))?;
                Literal::Opt(res.map(Box::new))
            }
            SBox => Literal::CBox(Arc::new(ErgoBox::sigma_parse(r)?).into()),
            SAvlTree => Literal::AvlTree(Box::new(AvlTreeData::sigma_parse(r)?)),
            SHeader if r.tree_version() >= ErgoTreeVersion::V3 => {
                Literal::Header(Box::new(Header::scorex_parse(r)?))
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
