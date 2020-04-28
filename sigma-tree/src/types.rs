#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
//! Sigma types

use core::num;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::{any::Any, io, marker::PhantomData, sync::Arc};
use SerializationError::InvalidTypePrefix;

#[derive(Clone, Debug)]
pub struct TypeCode(u8);
pub struct MethodId(u8);
pub struct TypeId(u8);

pub enum SType {
    SAny,
    SByte,
    SShort,
    SInt,
    SLong,
    SBigInt,
    SGroupElement,
    SSigmaProp,
    SBox,
    SAvlTree,
    SOption(Box<SType>),
    SColl(Box<SType>),
    STup(Vec<SType>),
    SFunc(Box<SFunc>),
}

impl SType {
    pub fn type_code(&self) -> TypeCode {
        todo!()
    }

    pub fn type_companion(&self) -> Option<STypeCompanion> {
        todo!()
    }
}

pub struct STypeVar {
    name: String,
}

pub struct STypeParam {
    ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}

pub struct SFunc {
    t_dom: Vec<SType>,
    t_range: SType,
    tpe_params: Vec<STypeParam>,
}

pub struct STypeCompanion {
    pub type_id: TypeId,
    pub type_name: String,
    pub methods: Vec<SMethod>,
}

pub struct SMethod {
    pub obj_type: Box<STypeCompanion>,
    pub name: String,
    pub method_id: MethodId,
    pub tpe: SType,
}

impl SigmaSerializable for SType {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L25-L25
        todo!()
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L118-L118
        let c = r.get_u8()?;
        if c == 0 {
            Err(InvalidTypePrefix)
        } else {
            todo!();
            // Ok(SType::SAny)
        }
    }
}
