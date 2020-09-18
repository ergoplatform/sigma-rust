//! Box value newtype

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::vlq_encode;
use std::{convert::TryFrom, io};
use thiserror::Error;

/// Box value
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct BoxValue(i64);

impl BoxValue {
    const MIN_RAW: i64 = 1;
    const MAX_RAW: i64 = i64::MAX;

    /// Minimal value
    pub const MIN: BoxValue = BoxValue(BoxValue::MIN_RAW);

    /// create from u64 with bounds check
    pub fn new(v: u64) -> Result<BoxValue, BoxValueError> {
        BoxValue::try_from(v)
    }

    /// Check if a value is in bounds
    pub fn within_bounds(v: u64) -> bool {
        v >= BoxValue::MIN_RAW as u64 && v <= BoxValue::MAX_RAW as u64
    }

    /// Get the value as u64
    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }

    /// Get the value as i64
    pub fn as_i64(&self) -> &i64 {
        &self.0
    }

    /// Addition with overflow check
    fn checked_add(&self, rhs: &Self) -> Result<Self, BoxValueError> {
        // TODO: add tests
        self.0
            .checked_add(rhs.0)
            .map(BoxValue)
            .ok_or(BoxValueError::Overflow)
    }

    /// Subtraction with overflow and bounds check
    pub fn checked_sub(self, rhs: Self) -> Result<Self, BoxValueError> {
        let raw_i64 = self.0.checked_sub(rhs.0).ok_or(BoxValueError::Overflow)?;
        if raw_i64 < BoxValue::MIN_RAW {
            Err(BoxValueError::OutOfBounds)
        } else {
            Ok(BoxValue(raw_i64))
        }
    }
}

impl PartialOrd for BoxValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other.as_i64())
    }
}

/// Sums up all iterator's box values
/// Returns Err on overflow
pub fn sum<I: Iterator<Item = BoxValue>>(mut iter: I) -> Result<BoxValue, BoxValueError> {
    // TODO: add tests (cover empty list)
    iter.try_fold(BoxValue(0), |acc, v| acc.checked_add(&v))
}

/// BoxValue errors
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum BoxValueError {
    /// Value is out of bounds
    #[error("Value is out of bounds")]
    OutOfBounds,
    /// Overflow
    #[error("Overflow")]
    Overflow,
}

impl TryFrom<u64> for BoxValue {
    type Error = BoxValueError;
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        if BoxValue::within_bounds(v) {
            Ok(BoxValue(v as i64))
        } else {
            Err(BoxValueError::OutOfBounds)
        }
    }
}

impl TryFrom<i64> for BoxValue {
    type Error = BoxValueError;
    fn try_from(v: i64) -> Result<Self, Self::Error> {
        if v >= BoxValue::MIN_RAW {
            Ok(BoxValue(v as i64))
        } else {
            Err(BoxValueError::OutOfBounds)
        }
    }
}

impl Into<u64> for BoxValue {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl Into<i64> for BoxValue {
    fn into(self) -> i64 {
        // it's safe since upper bound is i64::MAX
        self.0 as i64
    }
}

impl SigmaSerializable for BoxValue {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u64(self.0 as u64)
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let v = r.get_u64()?;
        Ok(BoxValue::try_from(v)?)
    }
}

impl From<BoxValueError> for SerializationError {
    fn from(e: BoxValueError) -> Self {
        SerializationError::ValueOutOfBounds(format!("{}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{arbitrary::Arbitrary, prelude::*};

    impl Arbitrary for BoxValue {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (BoxValue::MIN_RAW..BoxValue::MAX_RAW)
                .prop_map(BoxValue)
                .boxed()
        }
    }
}
