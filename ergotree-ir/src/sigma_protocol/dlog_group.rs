//! This is the general interface for the discrete logarithm prime-order group.
//!
//! The discrete logarithm problem is as follows: given a generator g of a finite
//! group G and a random element h in G, find the (unique) integer x such that
//! `g^x = h`.
//!
//! In cryptography, we are interested in groups for which the discrete logarithm problem
//! (Dlog for short) is assumed to be hard. The most known groups of that kind are some Elliptic curve groups.
//!
//! Another issue pertaining elliptic curves is the need to find a suitable mapping that will convert an arbitrary
//! message (that is some binary string) to an element of the group and vice-versa.
//!
//! Only a subset of the messages can be effectively mapped to a group element in such a way that there is a one-to-one
//! injection that converts the string to a group element and vice-versa.
//!
//! On the other hand, any group element can be mapped to some string.

use crate::bigint256::BigInt256;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::unsignedbigint256::UnsignedBigInt;
use bnum::types::{I256, U256};
use bnum::BTryFrom;
use elliptic_curve::PrimeField;
use k256::Scalar;
use num_traits::Num;
use rand_core::RngCore;
use sigma_ser::ScorexSerializable;

// /// Creates a random member of this Dlog group
// pub fn random_element() -> EcPoint {
//     let sk = DlogProverInput::random();
//     exponentiate(&generator(), &sk.w)
// }

/// Creates a random scalar, a big-endian integer in the range [0, n), where n is group order
/// Use cryptographically secure PRNG (like rand::thread_rng())
pub fn random_scalar_in_group_range(mut rng: impl RngCore) -> Scalar {
    let mut bytes = k256::FieldBytes::default();
    loop {
        rng.fill_bytes(&mut bytes);
        if let Some(scalar) = Scalar::from_repr(bytes).into() {
            return scalar;
        }
    }
}

/// Attempts to create BigInt256 from Scalar
/// Returns None if s > 2^255 - 1
/// Since Scalar is in [0, n) range, where n is the group order
/// (FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141)
/// it might not fit into 256-bit BigInt because BigInt uses 1 bit for sign.
pub fn scalar_to_bigint256(s: Scalar) -> Option<BigInt256> {
    // from https://github.com/RustCrypto/elliptic-curves/blob/fe737c56add103e4e8ff270d0c05ffdb6107b8d6/k256/src/arithmetic/scalar.rs#L598-L602
    let bytes = s.to_bytes();
    #[allow(clippy::unwrap_used)] // Scalar always fits in 256-bit unsigned integer
    let uint = U256::from_be_slice(&bytes).unwrap();
    <I256 as BTryFrom<U256>>::try_from(uint)
        .ok()
        .map(Into::into)
}

/// Attempts to create Scalar from BigInt256
pub fn bigint256_to_scalar(bi: BigInt256) -> Scalar {
    #[allow(clippy::unwrap_used)] // order != 0 so cannot fail
    UnsignedBigInt::from_signed_mod(bi, order()).unwrap().into()
}

impl SigmaSerializable for ergo_chain_types::EcPoint {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.scorex_serialize(w)?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let e = Self::scorex_parse(r)?;
        Ok(e)
    }
}

/// Order of the secp256k1 elliptic curve
// Since secp256k1 order doesn't fit in a signed 256 bit integer, this returns an unsigned 256-bit integer instead
#[allow(clippy::unwrap_used)]
pub fn order() -> UnsignedBigInt {
    UnsignedBigInt::from_str_radix(Scalar::MODULUS, 16).unwrap()
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
pub(crate) mod tests {
    use super::*;
    use elliptic_curve::PrimeField;
    use proptest::prelude::*;

    pub(crate) fn scalar() -> impl Strategy<Value = Scalar> {
        any::<[u8; 32]>().prop_filter_map(
            format!("Scalars must be 0 <= n < {0}", order()),
            |bytes| {
                if bytes[0] & 0x80 != 0 {
                    return None;
                }
                Scalar::from_repr(bytes.into()).into()
            },
        )
    }

    proptest! {
        #[test]
        fn scalar_biguint_roundtrip(scalar in scalar()) {
            let bu = scalar_to_bigint256(scalar).unwrap();
            let to_scalar = bigint256_to_scalar(bu);
            prop_assert_eq!(scalar, to_scalar);
        }

        #[test]
        fn scalar_bigint256_roundtrip(scalar in scalar()) {
            // Shift right to make sure that the MSB is 0, so that the Scalar can be
            // converted to a BigInt256
            let shifted_scalar = scalar >> 1;
            let as_bigint256: BigInt256 = scalar_to_bigint256(shifted_scalar).unwrap();
            let to_scalar = bigint256_to_scalar(as_bigint256);
            prop_assert_eq!(shifted_scalar, to_scalar);
        }
    }
}
