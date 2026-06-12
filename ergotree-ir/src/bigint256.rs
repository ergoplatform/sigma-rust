//! 256-bit signed integer type

use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::ops::{Div, Mul, Neg, Rem};

use bnum::cast::As;
use bnum::types::I256;
use bnum::BUintD8;
use derive_more::From;
use derive_more::{
    Add, AddAssign, BitAnd, BitOr, BitXor, Display, FromStr, Not, Shl, Shr, Sub, Sum,
};
use num_bigint::BigInt;
use num_derive::{Num, One, Signed, Zero};
use num_traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedShl, CheckedShr,
    CheckedSub, Signed, ToPrimitive,
};

use crate::serialization::{SigmaParsingError, SigmaSerializable};

/// 256-bit signed integer type
#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Display,
    From,
    FromStr,
    Copy,
    Clone,
    Zero,
    One,
    Num,
    Not,
    Add,
    AddAssign,
    Sub,
    BitAnd,
    BitOr,
    BitXor,
    Signed,
    Shl,
    Shr,
    Sum,
)]
pub struct BigInt256(pub(crate) bnum::types::I256);

impl BigInt256 {
    /// Create a BigInt256 from a slice of bytes in big-endian format. Returns None if slice.len() > 32 || slice.len() == 0
    pub fn from_be_slice(slice: &[u8]) -> Option<Self> {
        // match scala implementation which returns exception with empty byte array, whereas bnum returns 0
        if slice.is_empty() {
            return None;
        }
        I256::from_be_slice(slice).map(Self)
    }

    /// Return bytes of integer in big-endian order
    pub fn to_be_bytes(&self) -> [u8; 32] {
        *self
            .0
            .as_::<BUintD8<{ I256::BYTES as usize }>>()
            .to_be()
            .digits()
    }

    /// Convert BigInt256 to minimum number of bytes to represent it
    /// # Example
    /// ```
    /// # use ergotree_ir::bigint256::BigInt256;
    /// use num_traits::Num;
    ///
    /// let num = BigInt256::from_str_radix("ff", 16).unwrap();
    /// let num_bytes = num.to_be_vec();
    /// assert_eq!(num_bytes, vec![0x00, 0xff]);
    /// assert_eq!(num, BigInt256::from_be_slice(&num_bytes).unwrap());
    ///
    /// let neg = BigInt256::from_str_radix("-1", 16).unwrap();
    /// let neg_bytes = neg.to_be_vec();
    /// assert_eq!(neg_bytes, vec![0xff]);
    /// assert_eq!(neg, BigInt256::from_be_slice(&neg_bytes).unwrap());
    /// ```
    pub fn to_be_vec(&self) -> Vec<u8> {
        let mut bytes = self.0.to_radix_be(256);
        if self.0.is_negative() {
            // drain leading ones
            let leading_bytes = (self.0.leading_ones().saturating_sub(1)) / 8;
            bytes.drain(0..leading_bytes as usize);
        } else if bytes[0] & 0x80 != 0 {
            // If number has a leading 1, pad it with zeroes to avoid it being misinterpreted as negative by from_be_slice
            bytes.insert(0, 0);
        }
        bytes
    }
}

impl TryFrom<BigInt> for BigInt256 {
    type Error = String;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        let bytes = value.to_signed_bytes_be();
        Self::from_be_slice(&bytes).ok_or_else(|| "BigInt256 value: {value} out of bounds".into())
    }
}

impl From<BigInt256> for BigInt {
    fn from(value: BigInt256) -> Self {
        BigInt::from_signed_bytes_be(&value.to_be_bytes())
    }
}

impl From<i8> for BigInt256 {
    fn from(value: i8) -> Self {
        Self(I256::from(value))
    }
}

impl From<i16> for BigInt256 {
    fn from(value: i16) -> Self {
        Self(I256::from(value))
    }
}

impl From<i32> for BigInt256 {
    fn from(value: i32) -> Self {
        Self(I256::from(value))
    }
}

impl From<i64> for BigInt256 {
    fn from(value: i64) -> Self {
        Self(I256::from(value))
    }
}

impl Bounded for BigInt256 {
    fn min_value() -> Self {
        Self(I256::min_value())
    }

    fn max_value() -> Self {
        Self(I256::max_value())
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
        Some(BigInt256(self.0.checked_add(v.0)?))
    }
}

impl CheckedSub for BigInt256 {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        Some(BigInt256(self.0.checked_sub(v.0)?))
    }
}

impl CheckedMul for BigInt256 {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        Some(BigInt256(self.0.checked_mul(v.0)?))
    }
}

impl CheckedDiv for BigInt256 {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        Some(BigInt256(self.0.checked_div(v.0)?))
    }
}

impl CheckedRem for BigInt256 {
    fn checked_rem(&self, v: &Self) -> Option<Self> {
        // Scala BigInt does not allow modulo operations with negative divisors
        if v.is_negative() {
            return None;
        }
        self.0.checked_rem(v.0).map(Self)
    }
}

impl CheckedNeg for BigInt256 {
    fn checked_neg(&self) -> Option<Self> {
        if self == &BigInt256::min_value() {
            None
        } else {
            Some(-*self)
        }
    }
}

impl CheckedShl for BigInt256 {
    fn checked_shl(&self, rhs: u32) -> Option<Self> {
        self.0.checked_shl(rhs).map(Self)
    }
}

impl CheckedShr for BigInt256 {
    fn checked_shr(&self, rhs: u32) -> Option<Self> {
        self.0.checked_shr(rhs).map(Self)
    }
}

impl ToPrimitive for BigInt256 {
    fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.0.to_u64()
    }
}

impl SigmaSerializable for BigInt256 {
    fn sigma_serialize<W: crate::serialization::sigma_byte_writer::SigmaByteWrite>(
        &self,
        w: &mut W,
    ) -> crate::serialization::SigmaSerializeResult {
        let bytes = self.to_be_vec();
        w.put_u16(bytes.len() as u16)?;
        w.write_all(&bytes)?;
        Ok(())
    }

    fn sigma_parse<R: crate::serialization::sigma_byte_reader::SigmaByteRead>(
        r: &mut R,
    ) -> Result<Self, crate::serialization::SigmaParsingError> {
        let size = r.get_u16()?;
        if size > 32 {
            return Err(SigmaParsingError::ValueOutOfBounds(format!(
                "serialized BigInt size {0} bytes exceeds 32",
                size
            )));
        }
        let mut buf = vec![0u8; size as usize];
        r.get_bytes_into(&mut buf)?;
        match BigInt256::from_be_slice(&buf) {
            Some(x) => Ok(x),
            None => Err(SigmaParsingError::ValueOutOfBounds(String::new())),
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use proptest::{
        arbitrary::{any, Arbitrary},
        strategy::{BoxedStrategy, Strategy},
    };

    use super::BigInt256;

    impl Arbitrary for BigInt256 {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            #[allow(clippy::unwrap_used)]
            any::<[u8; 32]>()
                .prop_map(|bytes| Self::from_be_slice(&bytes[..]).unwrap())
                .boxed()
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Num;
    #[cfg(feature = "arbitrary")]
    use proptest::{prelude::*, proptest};

    #[cfg(feature = "arbitrary")]
    proptest! {
        #[test]
        fn roundtrip(b in any::<BigInt256>()) {
            let serialized = b.to_be_vec();
            assert_eq!(b, BigInt256::from_be_slice(&serialized).unwrap());
        }
        #[test]
        fn bigint_roundtrip(b in any::<BigInt256>()) {
            let bigint: BigInt = b.into();
            assert_eq!(b, bigint.try_into().unwrap());
        }
        #[test]
        fn upcast(l in any::<i64>()) {
            let bytes = l.to_be_bytes();
            let upcast = BigInt256::from(l);
            assert_eq!(upcast, BigInt256::from_be_slice(&bytes).unwrap());
        }
    }

    #[test]
    fn min_value() {
        let bigint_from_str = BigInt256::from_str_radix(
            "-57896044618658097711785492504343953926634992332820282019728792003956564819968",
            10,
        );
        assert_eq!(BigInt256::min_value(), bigint_from_str.unwrap());

        let mut bytes = [0x00_u8; 32];
        bytes[0] = 0x80;
        let bigint_from_bytes = BigInt256::from_be_slice(&bytes[..]);
        assert_eq!(BigInt256::min_value(), bigint_from_bytes.unwrap());

        let mut bytes = [0x00_u8; 33];
        bytes[0] = 0xff;
        bytes[1] = 0x80;
        let bigint_from_bytes = BigInt256::from_be_slice(&bytes[..]);
        assert_eq!(BigInt256::min_value(), bigint_from_bytes.unwrap());
    }

    #[test]
    fn max_value() {
        let bigint_from_str = BigInt256::from_str_radix(
            "57896044618658097711785492504343953926634992332820282019728792003956564819967",
            10,
        );
        assert_eq!(BigInt256::max_value(), bigint_from_str.unwrap());

        let mut bytes = [0xff_u8; 32];
        bytes[0] = 0x7f;
        let bigint_from_bytes = BigInt256::from_be_slice(&bytes[..]);
        assert_eq!(BigInt256::max_value(), bigint_from_bytes.unwrap());

        let mut bytes = [0xff_u8; 33];
        bytes[0] = 0x00;
        bytes[1] = 0x7f;
        let bigint_from_bytes = BigInt256::from_be_slice(&bytes[..]);
        assert_eq!(BigInt256::max_value(), bigint_from_bytes.unwrap());
    }

    #[test]
    fn out_of_bounds() {
        // Lower bound
        let bigint_from_str = BigInt256::from_str_radix(
            "-57896044618658097711785492504343953926634992332820282019728792003956564819969",
            10,
        );
        assert!(bigint_from_str.is_err());

        let mut bytes = [0xff_u8; 33];
        bytes[0] = 0xff;
        bytes[1] = 0x7f;
        let bigint_from_bytes = BigInt256::from_be_slice(&bytes[..]);
        assert!(bigint_from_bytes.is_none());

        // Upper bound
        let bigint_from_str = BigInt256::from_str_radix(
            "57896044618658097711785492504343953926634992332820282019728792003956564819968",
            10,
        );
        assert!(bigint_from_str.is_err());

        let mut bytes = [0x00_u8; 33];
        bytes[0] = 0x00;
        bytes[1] = 0x80;
        let bigint_from_bytes = BigInt256::from_be_slice(&bytes[..]);
        assert!(bigint_from_bytes.is_none());
    }
}
