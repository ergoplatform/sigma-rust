//! 256-bit unsigned big integer type
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::{Div, Mul, Rem};
use num_bigint::BigUint;

use bnum::{
    cast::{As, CastFrom},
    types::U256,
    BInt, BTryFrom, BUint, BUintD8,
};
use derive_more::{
    Add, AddAssign, BitAnd, BitOr, BitXor, Display, From, FromStr, Not, Shl, Shr, Sub, Sum,
};
use elliptic_curve::ops::Reduce;
use k256::Scalar;
use num_derive::{Num, One, ToPrimitive, Zero};
use num_traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedShl, CheckedShr, CheckedSub,
    Signed, Zero,
};

use crate::{
    bigint256::BigInt256,
    serialization::{SigmaParsingError, SigmaSerializable},
};

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
    ToPrimitive,
    Shl,
    Shr,
    Sum,
)]
/// Unsigned 256-bit integer type
pub struct UnsignedBigInt(U256);

impl UnsignedBigInt {
    /// Create a BigInt256 from a slice of bytes in big-endian format. Returns None if slice.len() > 32 || slice.len() == 0
    pub fn from_be_slice(slice: &[u8]) -> Option<Self> {
        U256::from_be_slice(slice).map(Self)
    }
    fn widen_to<const N: usize>(&self) -> BUint<N> {
        #[allow(clippy::unwrap_used)]
        // widen_to is only used for internal operations (modulo) and all of those will use widen_to with N > 4
        <BUint<N> as BTryFrom<U256>>::try_from(self.0).unwrap()
    }
    fn from_wide<const N: usize>(num: BUint<N>) -> Option<Self> {
        <U256 as BTryFrom<_>>::try_from(num).ok().map(Self)
    }

    /// Perform (self + other) % modulus. Returns None if modulus == 0
    pub fn checked_mod_add(&self, other: Self, modulus: Self) -> Option<Self> {
        // Perform addition using 320-bit result. Addition of two 256-bit ints can fit in 257 bits but using 5 * 64-bit ints should produce more efficient arithmetic on most machines
        let self_wide = self.widen_to::<5>();
        let other_wide = other.widen_to::<5>();
        let modulus_wide = modulus.widen_to::<5>();
        #[allow(clippy::unwrap_used)] // modulus < 2.pow(256) so this will never fail
        Some(Self::from_wide((self_wide + other_wide).checked_rem(modulus_wide)?).unwrap())
    }

    /// Calculate (self - other) % modulus
    pub fn checked_mod_sub(&self, other: Self, modulus: Self) -> Option<Self> {
        let other_inv = modulus.checked_sub(&(other % modulus))?;
        self.checked_mod_add(other_inv, modulus)
    }

    /// Perform (self * other) % modulus. Returns None if modulus == 0
    pub fn checked_mod_mul(&self, other: Self, modulus: Self) -> Option<Self> {
        let self_wide = self.widen_to::<8>();
        let other_wide = other.widen_to::<8>();
        let modulus_wide = modulus.widen_to::<8>();
        #[allow(clippy::unwrap_used)] // modulus < 2.pow(256) so this will never fail
        Some(Self::from_wide((self_wide * other_wide).checked_rem(modulus_wide)?).unwrap())
    }

    /// Compute modular inverse x such that (self * x) mod modulus = 1
    /// Returns None if modulus == 0 or inverse does not exist
    pub fn mod_inv(&self, modulus: Self) -> Option<Self> {
        /// Run the extended euclidean algorithm ax + by = gcd(a, b). Returns (gcd(a, b), x).
        fn extended_euclidean(a: UnsignedBigInt, b: UnsignedBigInt) -> (BInt<5>, BInt<5>) {
            let mut a = (a % b).widen_to().cast_signed();
            let mut b = b.widen_to().cast_signed();
            if b.is_zero() {
                return (a, 1.into());
            }
            let mut x0x1: (BInt<5>, BInt<5>) = (1.into(), 0.into());
            while !b.is_zero() {
                let q = a / b;
                x0x1 = (x0x1.1, x0x1.0 - q * x0x1.1);
                let tmp = a - q * b;
                a = b;
                b = tmp;
            }
            (a, x0x1.0)
        }
        let (g, x) = extended_euclidean(*self, modulus);
        if g.is_one() {
            let inv = x.checked_rem_euclid(modulus.widen_to().cast_signed())?;
            #[allow(clippy::unwrap_used)]
            // unwrap is used here for clarity, since inv is computed mod b, it will always fit in 256 bits and so this branch will always return Some(_)
            Some(Self::from_wide(inv.cast_unsigned()).unwrap())
        } else {
            None
        }
    }
    /// Convert UnsignedBigInt to minimum number of bytes to represent it
    /// # Example
    /// ```
    /// # use ergotree_ir::unsignedbigint256::UnsignedBigInt;
    /// use num_traits::Num;
    ///
    /// let num = UnsignedBigInt::from_str_radix("ff", 16).unwrap();
    /// let num_bytes = num.to_be_vec();
    /// assert_eq!(num_bytes, [0xff]);
    /// assert_eq!(num, UnsignedBigInt::from_be_slice(&num_bytes).unwrap());
    ///
    /// let neg = UnsignedBigInt::from_str_radix("0", 16).unwrap();
    /// let neg_bytes = neg.to_be_vec();
    /// assert_eq!(neg_bytes, Vec::<u8>::new());
    /// assert_eq!(neg, UnsignedBigInt::from_be_slice(&neg_bytes).unwrap());
    /// ```
    pub fn to_be_vec(&self) -> Vec<u8> {
        if self.is_zero() {
            vec![]
        } else {
            self.0.to_radix_be(256)
        }
    }

    /// Return bytes of integer in big-endian order
    pub fn to_be_bytes(&self) -> [u8; 32] {
        *self
            .0
            .as_::<BUintD8<{ U256::BYTES as usize }>>()
            .to_be()
            .digits()
    }

    /// Convert signed 256-bit integer to unsigned using euclidean remainder. The output will be >= 0 && < modulus. Returns None if modulus == 0
    pub fn from_signed_mod(signed: BigInt256, modulus: Self) -> Option<Self> {
        let signed_wide: BInt<5> = <BInt<5> as BTryFrom<BInt<4>>>::try_from(signed.0).ok()?;
        let modulus_wide: BInt<5> = modulus.widen_to().cast_signed();
        Self::from_wide(
            signed_wide
                .checked_rem_euclid(modulus_wide)?
                .cast_unsigned(),
        )
    }

    /// Create an `UnsignedBigInt` from limbs stored in little-endian order
    pub fn from_limbs(limbs: [u64; 4]) -> Self {
        Self(U256::from(limbs))
    }
    /// Convert `self` to underlying digits stored in little-endian order
    pub fn to_limbs(&self) -> [u64; 4] {
        *self.0.digits()
    }
}

impl TryFrom<BigUint> for UnsignedBigInt {
    type Error = String;

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes_be();
        Self::from_be_slice(&bytes).ok_or_else(|| "BigInt256 value: {value} out of bounds".into())
    }
}

impl From<u32> for UnsignedBigInt {
    fn from(value: u32) -> Self {
        Self(U256::from(value))
    }
}

impl From<u64> for UnsignedBigInt {
    fn from(value: u64) -> Self {
        Self(U256::from(value))
    }
}

impl From<UnsignedBigInt> for Scalar {
    fn from(value: UnsignedBigInt) -> Self {
        let bytes = *bnum::BUintD8::<32>::cast_from(value.0).to_be().digits();
        <Scalar as Reduce<k256::U256>>::reduce_bytes(&bytes.into())
    }
}

impl TryFrom<UnsignedBigInt> for BigInt256 {
    type Error = &'static str;

    fn try_from(value: UnsignedBigInt) -> Result<Self, Self::Error> {
        if value.0.bit(U256::BITS - 1) {
            Err("UnsignedBigInt out of bounds")
        } else {
            Ok(BigInt256(value.0.cast_signed()))
        }
    }
}

impl TryFrom<BigInt256> for UnsignedBigInt {
    type Error = &'static str;

    fn try_from(value: BigInt256) -> Result<Self, Self::Error> {
        if value.is_negative() {
            Err("Can not convert negative BigInt to UnsignedBigInt")
        } else {
            Ok(Self(value.0.cast_unsigned()))
        }
    }
}

impl CheckedAdd for UnsignedBigInt {
    fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
}

impl CheckedSub for UnsignedBigInt {
    fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
}

impl CheckedMul for UnsignedBigInt {
    fn checked_mul(&self, other: &Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }
}

impl CheckedDiv for UnsignedBigInt {
    fn checked_div(&self, other: &Self) -> Option<Self> {
        self.0.checked_div(other.0).map(Self)
    }
}

impl CheckedRem for UnsignedBigInt {
    fn checked_rem(&self, v: &Self) -> Option<Self> {
        self.0.checked_rem(v.0).map(Self)
    }
}

impl CheckedShl for UnsignedBigInt {
    fn checked_shl(&self, rhs: u32) -> Option<Self> {
        self.0.checked_shl(rhs).map(Self)
    }
}

impl CheckedShr for UnsignedBigInt {
    fn checked_shr(&self, rhs: u32) -> Option<Self> {
        self.0.checked_shr(rhs).map(Self)
    }
}

impl Bounded for UnsignedBigInt {
    fn min_value() -> Self {
        Self(U256::min_value())
    }

    fn max_value() -> Self {
        Self(U256::max_value())
    }
}

impl Mul for UnsignedBigInt {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        UnsignedBigInt(self.0 * rhs.0)
    }
}

impl Div for UnsignedBigInt {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        UnsignedBigInt(self.0 / rhs.0)
    }
}

impl Rem for UnsignedBigInt {
    type Output = UnsignedBigInt;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl SigmaSerializable for UnsignedBigInt {
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
        let size = r.get_u16()? as usize;
        if size > 32 {
            return Err(SigmaParsingError::ValueOutOfBounds(format!(
                "serialized BigInt size {0} bytes exceeds 32",
                size
            )));
        }
        let mut buf = [0u8; 32];
        r.get_bytes_into(&mut buf[32 - size..])?;
        match UnsignedBigInt::from_be_slice(&buf) {
            Some(x) => Ok(x),
            None => Err(SigmaParsingError::ValueOutOfBounds("".into())),
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use proptest::{
        arbitrary::{any, Arbitrary},
        strategy::{BoxedStrategy, Strategy},
    };

    use super::UnsignedBigInt;

    impl Arbitrary for UnsignedBigInt {
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

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod test {
    use k256::Scalar;
    use num_bigint::BigInt;
    use num_traits::{CheckedEuclid, Euclid, Num, Zero};
    use proptest::prelude::*;

    use crate::{
        bigint256::BigInt256,
        ergo_tree::ErgoTreeVersion,
        mir::constant::Constant,
        serialization::{roundtrip_new_feature, SigmaSerializable},
    };

    use super::UnsignedBigInt;
    // Inefficient impl of UnsignedBigInt -> BigInt, this is only used for tests so should be acceptable
    fn to_bigint(num: UnsignedBigInt) -> BigInt {
        BigInt::from_str_radix(&num.to_string(), 10).unwrap()
    }
    #[test]
    fn serialize_zero() {
        // zero is serialized as (length == 0, [])
        assert_eq!(
            UnsignedBigInt::from(0u32).sigma_serialize_bytes().unwrap(),
            [0]
        );
        assert!(UnsignedBigInt::sigma_parse_bytes(&[0]).unwrap().is_zero());
    }
    proptest! {
        #[test]
        fn to_scalar(s in crate::sigma_protocol::dlog_group::tests::scalar()) {
            let bytes = s.to_bytes();
            let bigint = UnsignedBigInt::from_be_slice(&bytes[..]).unwrap();
            assert_eq!(Scalar::from(bigint), s);
        }

        #[test]
        fn ser_roundtrip(v in any::<UnsignedBigInt>()) {
            let v = Constant::from(v);
            roundtrip_new_feature(&v, ErgoTreeVersion::V3);
        }
        #[test]
        fn mod_add(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>(), c in any::<UnsignedBigInt>()) {
            let a_bigint = to_bigint(a);
            let b_bigint = to_bigint(b);
            let c_bigint = to_bigint(c);
            let res = a.checked_mod_add(b, c);
            if c != UnsignedBigInt::zero() {
                assert_eq!(to_bigint(res.unwrap()), ((a_bigint + b_bigint) % c_bigint));
            }
            else {
                assert!(res.is_none());
            }
        }
        #[test]
        fn mod_sub(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>(), c in any::<UnsignedBigInt>()) {
            let a_bigint = to_bigint(a);
            let b_bigint = to_bigint(b);
            let c_bigint = to_bigint(c);
            let res = a.checked_mod_sub(b, c);
            if c != UnsignedBigInt::zero() {
                assert_eq!(to_bigint(res.unwrap()), ((a_bigint - b_bigint).rem_euclid(&c_bigint)));
            }
            else {
                assert!(res.is_none());
            }
        }
        #[test]
        fn mod_mul(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>(), c in any::<UnsignedBigInt>()) {
            let a_bigint = to_bigint(a);
            let b_bigint = to_bigint(b);
            let c_bigint = to_bigint(c);
            let res = a.checked_mod_mul(b, c);
            if c != UnsignedBigInt::zero() {
                assert_eq!(to_bigint(res.unwrap()), (a_bigint * b_bigint) % c_bigint);
            }
            else {
                assert!(res.is_none());
            }
        }
        #[test]
        fn mod_inv(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>()) {
            let a_bigint = to_bigint(a);
            let b_bigint = to_bigint(b);
            let inverse = a.mod_inv(b);
            if let Some(inverse) = inverse {
                let inverse_bigint = a_bigint.modinv(&b_bigint);
                assert_eq!(a.checked_mod_mul(inverse, b).unwrap(), 1u32.into());
                assert_eq!(to_bigint(inverse), inverse_bigint.unwrap())
            }
            else if !b.is_zero() {
                assert!(a_bigint.modinv(&b_bigint).is_none());
            }
        }
        #[test]
        fn to_unsigned_mod(a in any::<BigInt256>(), modulus in any::<UnsignedBigInt>()) {
            let a_bigint = BigInt::from_str_radix(&a.to_string(), 10).unwrap();
            let modulus_bigint = to_bigint(modulus);
            assert_eq!(UnsignedBigInt::from_signed_mod(a, modulus).map(to_bigint), a_bigint.checked_rem_euclid(&modulus_bigint));
        }
    }
}
