use crate::data::{SigmaBox, SigmaProp};
use std::ops::Add;

pub enum Value {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    BigInt,       // TODO: find underlying type
    GroupElement, // TODO: find/make underlying type
    SigmaProp(Box<dyn SigmaProp>),
    Box(Box<dyn SigmaBox>),
    AvlTree, // TODO: make underlying type
    Coll(Vec<Value>),
    Tup(Vec<Value>),
}

impl Value {
    #[inline]
    pub fn byte(value: i8) -> Self {
        Value::Byte(value)
    }
}

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Value::Byte(s), Value::Byte(o)) => Self::byte(s + o),
            _ => todo!(),
        }
    }
}
