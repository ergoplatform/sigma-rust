use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Deref, Div, Mul, Neg, Rem, Sub};

use num256::int256::Int256;
use num_bigint::BigInt;
use num_derive::{One, Zero};
use num_traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub, Num};

/// 256-bit integer type
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Zero, One)]
pub struct BigInt256(Int256);

impl TryFrom<BigInt> for BigInt256 {
    type Error = String;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        if value < Self::min_value().0 .0 {
            Err(format!("BigInt256: Value {} is smaller than -2^255", value))
        } else if value > Self::max_value().0 .0 {
            Err(format!("BigInt256: Value {} is larger than 2^255-1", value))
        } else {
            Ok(Self(Int256(value)))
        }
    }
}

impl TryFrom<&[u8]> for BigInt256 {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let n = BigInt::from_signed_bytes_be(value);
        Self::try_from(n)
    }
}

impl TryFrom<i8> for BigInt256 {
    type Error = String;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        let n = BigInt::from(value);
        Self::try_from(n)
    }
}

impl TryFrom<i16> for BigInt256 {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        let n = BigInt::from(value);
        Self::try_from(n)
    }
}

impl TryFrom<i32> for BigInt256 {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let n = BigInt::from(value);
        Self::try_from(n)
    }
}

impl TryFrom<i64> for BigInt256 {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let n = BigInt::from(value);
        Self::try_from(n)
    }
}

impl Deref for BigInt256 {
    type Target = Int256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Num for BigInt256 {
    type FromStrRadixErr = <Int256 as Num>::FromStrRadixErr;

    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self(Int256::from_str_radix(s, radix)?))
    }
}

impl Bounded for BigInt256 {
    fn min_value() -> Self {
        Self(Int256::min_value())
    }

    fn max_value() -> Self {
        Self(Int256::max_value())
    }
}

impl Add for BigInt256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        BigInt256(self.0 + rhs.0)
    }
}

impl Sub for BigInt256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        BigInt256(self.0 - rhs.0)
    }
}

impl Mul for BigInt256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        BigInt256(self.0 * rhs.0)
    }
}

impl Div for BigInt256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        BigInt256(self.0 / rhs.0)
    }
}

impl Rem for BigInt256 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        BigInt256(self.0 % rhs.0)
    }
}

impl Neg for BigInt256 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        BigInt256(-self.0)
    }
}

impl CheckedAdd for BigInt256 {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        Some(BigInt256(self.0.checked_add(&v.0)?))
    }
}

impl CheckedSub for BigInt256 {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        Some(BigInt256(self.0.checked_sub(&v.0)?))
    }
}

impl CheckedMul for BigInt256 {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        Some(BigInt256(self.0.checked_mul(&v.0)?))
    }
}

impl CheckedDiv for BigInt256 {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        Some(BigInt256(self.0.checked_div(&v.0)?))
    }
}

impl CheckedNeg for BigInt256 {
    fn checked_neg(&self) -> Option<Self> {
        if self == &BigInt256::min_value() {
            None
        } else {
            Some(-self.clone())
        }
    }
}

impl BitAnd for BigInt256 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BigInt256(Int256(self.0 .0 & rhs.0 .0))
    }
}

impl<'a> BitAnd<&'a BigInt256> for &'a BigInt256 {
    type Output = BigInt256;

    fn bitand(self, rhs: &BigInt256) -> Self::Output {
        BigInt256(Int256(&self.0 .0 & &rhs.0 .0))
    }
}

impl BitOr for BigInt256 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        BigInt256(Int256(self.0 .0 | rhs.0 .0))
    }
}

impl<'a> BitOr<&'a BigInt256> for &'a BigInt256 {
    type Output = BigInt256;

    fn bitor(self, rhs: &BigInt256) -> Self::Output {
        BigInt256(Int256(&self.0 .0 | &rhs.0 .0))
    }
}

impl BitXor for BigInt256 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BigInt256(Int256(self.0 .0 ^ rhs.0 .0))
    }
}

impl<'a> BitXor<&'a BigInt256> for &'a BigInt256 {
    type Output = BigInt256;

    fn bitxor(self, rhs: &BigInt256) -> Self::Output {
        BigInt256(Int256(&self.0 .0 ^ &rhs.0 .0))
    }
}

impl fmt::Display for BigInt256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.to_str_radix(10))
    }
}
