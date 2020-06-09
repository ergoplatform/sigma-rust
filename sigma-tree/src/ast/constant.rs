use crate::{chain::ErgoBox, data::SigmaProp, types::SType};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CollPrim {
    CollBoolean(Vec<bool>),
    CollByte(Vec<i8>),
    CollShort(Vec<i16>),
    CollInt(Vec<i32>),
    CollLong(Vec<i64>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ConstantVal {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    BigInt,
    GroupElement,
    SigmaProp(Box<SigmaProp>),
    CBox(Box<ErgoBox>),
    AvlTree,
    CollPrim(CollPrim),
    Coll(Vec<ConstantVal>),
    Tup(Vec<ConstantVal>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Constant {
    pub tpe: SType,
    pub v: ConstantVal,
}

impl Constant {
    pub fn sigma_prop(prop: SigmaProp) -> Constant {
        Constant {
            tpe: SType::SSigmaProp,
            v: ConstantVal::SigmaProp(Box::new(prop)),
        }
    }
}
