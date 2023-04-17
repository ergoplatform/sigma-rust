//! Box value newtype

use crate::chain::token::TokenAmountError;
use crate::mir::constant::Constant;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use derive_more::FromStr;
use std::convert::TryFrom;
use thiserror::Error;

#[cfg(not(feature = "json"))]
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, FromStr)]
/// Box value in nanoERGs with bound checks
pub struct BoxValue(pub(crate) u64);

#[cfg(feature = "json")]
#[derive(
    serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy, FromStr,
)]
#[serde(
    try_from = "crate::chain::json::box_value::BoxValueJson",
    into = "crate::chain::json::box_value::BoxValueJson"
)]
/// Box value in nanoERGs with bound checks
pub struct BoxValue(pub(crate) u64);

impl BoxValue {
    /// Minimal box value per byte of the serialized box that was set on on launch
    pub const MIN_VALUE_PER_BOX_BYTE: u32 = 360;
    /// Minimal theoretical box size (smallest tree, no tokens, no registers, etc.)
    const MIN_BOX_SIZE_BYTES: usize = 30;

    /// Absolute minimal value, calculated from smallest possible box size and original value per byte requirement
    pub const MIN_RAW: u64 =
        BoxValue::MIN_VALUE_PER_BOX_BYTE as u64 * BoxValue::MIN_BOX_SIZE_BYTES as u64;
    /// Absolute maximal allowed box value
    pub const MAX_RAW: u64 = i64::MAX as u64;

    /// Absolute minimal value, calculated from smallest possible box size and original value per byte requirement
    pub const MIN: BoxValue = BoxValue(BoxValue::MIN_RAW);

    /// Recommended (safe) minimal box value to use in case box size estimation is unavailable.
    /// Allows box size up to 2777 bytes with current min box value per byte of 360 nanoERGs
    pub const SAFE_USER_MIN: BoxValue = BoxValue(1000000);

    /// Number of units inside one ERGO (i.e. one ERG using nano ERG representation)
    pub const UNITS_PER_ERGO: u32 = 1000000000;

    /// create from u64 with bounds check
    pub fn new(v: u64) -> Result<BoxValue, BoxValueError> {
        BoxValue::try_from(v)
    }

    /// Check if a value is in bounds
    pub const fn within_bounds(v: u64) -> bool {
        v >= BoxValue::MIN_RAW && v <= BoxValue::MAX_RAW
    }

    /// Get the value as u64
    pub fn as_u64(&self) -> &u64 {
        &self.0
    }

    /// Get the value as i64
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }

    /// Addition with overflow check
    pub fn checked_add(&self, rhs: &Self) -> Result<Self, BoxValueError> {
        let raw = self.0.checked_add(rhs.0).ok_or(BoxValueError::Overflow)?;
        if raw > Self::MAX_RAW {
            Err(BoxValueError::OutOfBounds(raw))
        } else {
            Ok(Self(raw))
        }
    }

    /// Subtraction with overflow and bounds check
    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, BoxValueError> {
        let raw = self.0.checked_sub(rhs.0).ok_or(BoxValueError::Overflow)?;
        if raw < Self::MIN_RAW {
            Err(BoxValueError::OutOfBounds(raw))
        } else {
            Ok(Self(raw))
        }
    }

    /// Multiplication with overflow check
    pub fn checked_mul(&self, rhs: &Self) -> Result<Self, BoxValueError> {
        let raw = self.0.checked_mul(rhs.0).ok_or(BoxValueError::Overflow)?;
        if raw > Self::MAX_RAW {
            Err(BoxValueError::OutOfBounds(raw))
        } else {
            Ok(Self(raw))
        }
    }

    /// Multiplication with overflow check
    pub fn checked_mul_u32(&self, rhs: u32) -> Result<Self, BoxValueError> {
        let raw = self
            .0
            .checked_mul(rhs as u64)
            .ok_or(BoxValueError::Overflow)?;
        if raw > Self::MAX_RAW {
            Err(BoxValueError::OutOfBounds(raw))
        } else {
            Ok(Self(raw))
        }
    }
}

impl PartialOrd for BoxValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other.as_u64())
    }
}

impl TryFrom<u64> for BoxValue {
    type Error = BoxValueError;
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        if BoxValue::within_bounds(v) {
            Ok(BoxValue(v))
        } else {
            Err(BoxValueError::OutOfBounds(v))
        }
    }
}

impl TryFrom<i64> for BoxValue {
    type Error = BoxValueError;
    fn try_from(v: i64) -> Result<Self, Self::Error> {
        if v >= BoxValue::MIN_RAW as i64 {
            Ok(BoxValue(v as u64))
        } else {
            Err(BoxValueError::OutOfBounds(v as u64))
        }
    }
}

impl From<BoxValue> for u64 {
    fn from(v: BoxValue) -> Self {
        v.0
    }
}

impl From<BoxValue> for i64 {
    fn from(v: BoxValue) -> Self {
        // it's safe since upper bound is i64::MAX
        v.0 as i64
    }
}

impl From<BoxValue> for Constant {
    fn from(v: BoxValue) -> Self {
        v.as_i64().into()
    }
}

impl SigmaSerializable for BoxValue {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u64(self.0)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let v = r.get_u64()?;
        Ok(BoxValue::try_from(v)?)
    }
}

/// BoxValue errors
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum BoxValueError {
    /// Value is out of bounds
    #[error("Box value is out of bounds: {0}")]
    OutOfBounds(u64),
    /// Overflow
    #[error("Overflow")]
    Overflow,
}

impl From<BoxValueError> for SigmaParsingError {
    fn from(e: BoxValueError) -> Self {
        SigmaParsingError::ValueOutOfBounds(format!("BoxValue error: {}", e))
    }
}

impl From<TokenAmountError> for SigmaParsingError {
    fn from(e: TokenAmountError) -> Self {
        SigmaParsingError::ValueOutOfBounds(format!("TokenAmount error: {}", e))
    }
}

/// Sums up all iterator's box values
/// Returns Err on overflow
pub fn checked_sum<I: Iterator<Item = BoxValue>>(mut iter: I) -> Result<BoxValue, BoxValueError> {
    iter.try_fold(BoxValue(0), |acc, v| acc.checked_add(&v))
        .map_or_else(Err, |v| {
            if v.0 == 0 {
                // input list was empty (sum is zero)
                Err(BoxValueError::OutOfBounds(0))
            } else {
                Ok(v)
            }
        })
}

/// Arbitrary
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use derive_more::{From, Into};
    use proptest::{arbitrary::Arbitrary, prelude::*};
    use std::ops::Range;

    use super::*;

    /// Range for arbitrary BoxValue values
    #[derive(Debug, From, Into)]
    pub struct ArbBoxValueRange(Range<u64>);

    impl Default for ArbBoxValueRange {
        fn default() -> Self {
            ArbBoxValueRange(BoxValue::MIN_RAW..(BoxValue::MAX_RAW / 10))
        }
    }

    impl Arbitrary for BoxValue {
        type Parameters = ArbBoxValueRange;
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (args.0).prop_map(BoxValue).boxed()
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
pub mod tests {
    use std::convert::TryInto;

    use super::*;
    use proptest::{collection::vec, prelude::*};

    extern crate derive_more;

    #[test]
    fn test_checked_add() {
        let a = BoxValue::try_from(10000000u64)
            .unwrap()
            .checked_add(&10000000u64.try_into().unwrap())
            .unwrap();
        assert_eq!(a, 20000000u64.try_into().unwrap());
    }

    #[test]
    fn test_checked_add_overflow() {
        assert!(BoxValue::try_from(BoxValue::MAX_RAW)
            .unwrap()
            .checked_add(&BoxValue::MIN)
            .is_err())
    }

    #[test]
    fn test_checked_sub() {
        let a = BoxValue::try_from(10000000u64)
            .unwrap()
            .checked_sub(&5000000u64.try_into().unwrap())
            .unwrap();
        assert_eq!(a, 5000000u64.try_into().unwrap());
    }

    #[test]
    fn test_checked_sub_overflow() {
        assert!(BoxValue::MIN
            .checked_sub(&BoxValue::MAX_RAW.try_into().unwrap())
            .is_err())
    }

    #[test]
    fn test_checked_mul() {
        let a = BoxValue::MIN.checked_mul(&BoxValue::MIN).unwrap();
        assert_eq!(
            a,
            (BoxValue::MIN_RAW * BoxValue::MIN_RAW).try_into().unwrap()
        );
    }

    #[test]
    fn test_checked_mul_overflow() {
        assert!(
            BoxValue::try_from(BoxValue::MAX_RAW / BoxValue::MIN_RAW + 1)
                .unwrap()
                .checked_mul(&BoxValue::MIN)
                .is_err()
        )
    }

    #[test]
    fn test_checked_mul_u32_overflow() {
        assert!(
            BoxValue::try_from(BoxValue::MAX_RAW / BoxValue::MIN_RAW + 1)
                .unwrap()
                .checked_mul_u32(BoxValue::MIN_RAW as u32)
                .is_err()
        )
    }

    #[test]
    fn test_checked_sum_empty_input() {
        let empty_input: Vec<BoxValue> = vec![];
        assert!(checked_sum(empty_input.into_iter()).is_err());
    }

    #[test]
    fn test_checked_sum_overflow() {
        let input: Vec<BoxValue> = vec![BoxValue::MAX_RAW.try_into().unwrap(), BoxValue::MIN];
        assert!(checked_sum(input.into_iter()).is_err());
    }

    proptest! {

        #[test]
        fn test_checked_sum(inputs in vec(any_with::<BoxValue>((9000..10000000).into()), 1..10)) {
            let expected_sum: u64 = inputs.clone().into_iter().map(|v| *v.as_u64()).sum();
            let checked_sum = checked_sum(inputs.into_iter()).unwrap();
            assert_eq!(*checked_sum.as_u64(), expected_sum);
        }
    }
}
