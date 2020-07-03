//! Box value newtype

#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::serializer::{SerializationError, SigmaSerializable};
use sigma_ser::vlq_encode;
use std::{convert::TryFrom, io};

/// Box value
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct BoxValue(u64);

impl BoxValue {
    const MIN_RAW: u64 = 1;
    const MAX_RAW: u64 = std::i64::MAX as u64;

    /// Minimal value
    pub const MIN: BoxValue = BoxValue(BoxValue::MIN_RAW);

    /// Create from u64 with bounds check
    pub fn new(v: u64) -> Result<BoxValue, BoxValueError> {
        BoxValue::try_from(v)
    }

    /// Check if a value is in bounds
    pub fn within_bounds(v: u64) -> bool {
        v >= BoxValue::MIN_RAW && v <= BoxValue::MAX_RAW
    }

    /// Get u64 value
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// BoxValue errors
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BoxValueError {
    /// Value is out of bounds
    OutOfBounds,
}

impl TryFrom<u64> for BoxValue {
    type Error = BoxValueError;
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        if BoxValue::within_bounds(v) {
            Ok(BoxValue(v))
        } else {
            Err(BoxValueError::OutOfBounds)
        }
    }
}

impl Into<u64> for BoxValue {
    fn into(self) -> u64 {
        self.0
    }
}

impl SigmaSerializable for BoxValue {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u64(self.0)
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let v = r.get_u64()?;
        Ok(BoxValue(v))
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
            (1..i64::MAX).prop_map(|v| BoxValue(v as u64)).boxed()
        }
    }
}
