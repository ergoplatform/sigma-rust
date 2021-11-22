// TODO: remove after all todo! are implemented
#![allow(clippy::todo)]

use super::challenge::Challenge;
use super::gf2_192::Gf2_192;

#[derive(PartialEq, Debug, Clone)]
pub struct Gf2_192Poly {}

impl Gf2_192Poly {
    pub(crate) fn from_byte_array(_challenge: Challenge, _: Vec<u8>) -> Self {
        todo!()
    }

    pub(crate) fn evaluate(&self, _idx: usize) -> Gf2_192 {
        todo!()
    }

    pub(crate) fn interpolate(
        _points: Vec<u8>,
        _values: Vec<Gf2_192>,
        _value_at_zero: Gf2_192,
    ) -> Gf2_192Poly {
        todo!()
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}
