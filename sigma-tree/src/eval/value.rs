use crate::data::{SigmaBox, SigmaProp};
use std::ops::Add;

#[allow(dead_code)]
pub enum Value {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    BigInt,
    GroupElement,
    SigmaProp(Box<dyn SigmaProp>),
    Box(Box<dyn SigmaBox>),
    AvlTree,
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
