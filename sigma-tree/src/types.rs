#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
//! Sigma types

use crate::ast::{ConstantNode, Value};
use core::num;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::{any::Any, io, marker::PhantomData, sync::Arc};

#[derive(Clone, Debug)]
pub struct TypeCode(u8);
pub struct MethodId(u8);
pub struct TypeId(u8);

// pub trait STypeTrait {
//     fn kind(&self) -> STypeKind;
//     fn code(&self) -> TypeCode;
// }

// pub struct SType {
//     kind: Arc<STypeKind>,
//     code: TypeCode,
// }

// pub struct SOption<T> {
//     pub elem_type: T,
// }

// pub struct SCollection<T> {
//     pub elem_type: T,
// }

// pub struct STuple {
//     items: Vec<SType>,
// }

// pub struct SSigmaProp {}

// pub struct SInt {}
// pub struct SBox {}

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
}

impl SType {
    pub fn type_code() -> TypeCode {
        todo!()
    }
}

impl SigmaSerializable for SType {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L25-L25
        todo!()
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: R) -> Result<Self, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L118-L118
        todo!()
    }
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

pub enum SigmaBoolean {
    ProveDlog(u64),
    CAND(Vec<SigmaBoolean>),
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

pub trait STypeCompanion {
    fn type_id(&self) -> TypeId;
    fn type_name(&self) -> &'static str;
}

impl STypeCompanion for SType {
    fn type_id(&self) -> TypeId {
        todo!()
    }
    fn type_name(&self) -> &'static str {
        todo!()
    }
}

pub struct SMethod {
    pub obj_type: Box<dyn STypeCompanion>,
    pub name: String,
    pub method_id: MethodId,
    pub tpe: SType,
}

pub fn deserialize_type() -> SType {
    todo!()
}
