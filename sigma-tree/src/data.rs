#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
//! Sigma data
use crate::{ast::Expr, types::SType};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::{any::Any, io};
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use ConstantKind::*;
use SType::*;

pub struct RegisterId(u8);

pub enum CCollPrim {
    CCollBoolean(Vec<bool>),
    CCollByte(Vec<i8>),
    CCollShort(Vec<i16>),
    CCollInt(Vec<i32>),
    CCollLong(Vec<i64>),
}

pub enum ConstantKind {
    CBoolean(bool),
    CByte(i8),
    CShort(i16),
    CInt(i32),
    CLong(i64),
    CBigInt,       // TODO: find underlying type
    CGroupElement, // TODO: find/make underlying type
    CSigmaProp(Box<dyn SigmaProp>),
    CBox(Box<dyn SigmaBox>),
    CAvlTree, // TODO: make underlying type
    CCollPrim(CCollPrim),
    CColl(Vec<ConstantKind>),
    CTup(Vec<ConstantKind>),
}

pub enum SigmaBoolean {
    ProveDlog(u64),
    CAND(Vec<SigmaBoolean>),
}

pub trait SigmaProp {
    fn is_valid(&self) -> bool;
}

pub struct CSigmaProp {
    pub sigma_tree: SigmaBoolean,
}

impl SigmaProp for CSigmaProp {
    fn is_valid(&self) -> bool {
        todo!()
    }
}

pub trait SigmaBox {
    fn value(&self) -> u64;
}
pub struct CSigmaBox {}
impl SigmaBox for CSigmaBox {
    fn value(&self) -> u64 {
        0
    }
}

pub fn sigma_serialize_const<W: WriteSigmaVlqExt>(
    c: ConstantKind,
    tpe: SType,
    w: W,
) -> Result<(), io::Error> {
    // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L26-L26
    todo!()
}

pub fn sigma_deserialize_const<R: ReadSigmaVlqExt>(
    tpe: &SType,
    mut r: R,
) -> Result<ConstantKind, SerializationError> {
    // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
    let c = match tpe {
        SAny => todo!(),
        SByte => CByte(r.get_i8()?),
        SColl(et) => {
            let len = r.get_u16()? as usize;
            if **et == SByte {
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
            types
                .iter()
                .try_for_each(|tpe| sigma_deserialize_const(tpe, &mut r).map(|v| items.push(v)))?;
            CTup(items)
        }

        _ => todo!("handle the rest of the constant types"),
    };
    Ok(c)
}
