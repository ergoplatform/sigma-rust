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
use elliptic_curve::rand_core::RngCore;
use k256::elliptic_curve::PrimeField;
use k256::Scalar;
use num_bigint::Sign;
use num_bigint::ToBigUint;
use num_bigint::{BigInt, BigUint};
use num_traits::ToPrimitive;
use sigma_ser::ScorexSerializable;
use std::convert::TryFrom;

// /// Creates a random member of this Dlog group
// pub fn random_element() -> EcPoint {
//     let sk = DlogProverInput::random();
//     exponentiate(&generator(), &sk.w)
// }

/// Creates a random scalar, a big-endian integer in the range [0, n), where n is group order
/// Use cryptographically secure PRNG (like rand::thread_rng())
pub fn random_scalar_in_group_range(mut rng: impl RngCore) -> Scalar {
    Scalar::generate_vartime(&mut rng)
}

/// Attempts to create BigInt256 from Scalar
/// Returns None if s > 2^255 - 1
/// Since Scalar is in [0, n) range, where n is the group order
/// (FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141)
/// it might not fit into 256-bit BigInt because BigInt uses 1 bit for sign.
pub fn scalar_to_bigint256(s: Scalar) -> Option<BigInt256> {
    // from https://github.com/RustCrypto/elliptic-curves/blob/fe737c56add103e4e8ff270d0c05ffdb6107b8d6/k256/src/arithmetic/scalar.rs#L598-L602
    let bytes = s.to_bytes();
    #[allow(clippy::unwrap_used)]
    let bu: BigUint = bytes
        .iter()
        .enumerate()
        .map(|(i, w)| w.to_biguint().unwrap() << ((31 - i) * 8))
        .sum();
    BigInt256::try_from(bu).ok()
}

fn biguint_to_bytes(x: &BigUint) -> [u8; 32] {
    // from https://github.com/RustCrypto/elliptic-curves/blob/fe737c56add103e4e8ff270d0c05ffdb6107b8d6/k256/src/arithmetic/scalar.rs#L587-L588
    let mask = BigUint::from(u8::MAX);
    let mut bytes = [0u8; 32];
    #[allow(clippy::needless_range_loop)]
    #[allow(clippy::unwrap_used)]
    for i in 0..32 {
        bytes[i] = ((x >> ((31 - i) * 8)) as BigUint & &mask).to_u8().unwrap();
    }
    bytes
}

/// Attempts to create Scalar from BigInt256
/// Returns None if not in the range [0, modulus).
pub fn bigint256_to_scalar(bi: BigInt256) -> Option<Scalar> {
    if Sign::Minus == bi.sign() {
        return None;
    }
    #[allow(clippy::unwrap_used)] // since it's 256-bit BigInt it should always fit into BigUint
    let bu = bi.to_biguint().unwrap();
    let bytes = biguint_to_bytes(&bu);
    Scalar::from_repr(bytes.into()).into()
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
pub fn order() -> BigInt {
    #[allow(clippy::unwrap_used)]
    BigInt::parse_bytes(
        b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
        16,
    )
    .unwrap()
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use num_bigint::ToBigUint;
    use proptest::prelude::*;

    // the following Scalar <-> BigUint helpers are from k256::arithmetic::scalar

    /// Converts a byte array (big-endian) to BigUint.
    fn bytes_to_biguint(bytes: &[u8; 32]) -> BigUint {
        bytes
            .iter()
            .enumerate()
            .map(|(i, w)| w.to_biguint().unwrap() << ((31 - i) * 8))
            .sum()
    }

    fn scalar_to_biguint(scalar: &Scalar) -> Option<BigUint> {
        Some(bytes_to_biguint(scalar.to_bytes().as_ref()))
    }

    fn biguint_to_scalar(x: &BigUint) -> Scalar {
        debug_assert!(x < &modulus_as_biguint());
        let bytes = biguint_to_bytes(x);
        Scalar::from_repr(bytes.into()).unwrap()
    }

    /// Returns the scalar modulus as a `BigUint` object.
    fn modulus_as_biguint() -> BigUint {
        scalar_to_biguint(&Scalar::ONE.negate()).unwrap() + 1.to_biguint().unwrap()
    }

    prop_compose! {
        fn scalar()(bytes in any::<[u8; 32]>()) -> Scalar {
            let mut res = bytes_to_biguint(&bytes);
            let m = modulus_as_biguint();
            // Modulus is 256 bit long, same as the maximum `res`,
            // so this is guaranteed to land us in the correct range.
            if res >= m {
                res -= m;
            }
            biguint_to_scalar(&res)
        }
    }

    proptest! {

        #[test]
        fn scalar_biguint_roundtrip(scalar in scalar()) {
            let bu = scalar_to_biguint(&scalar).unwrap();
            let to_scalar = biguint_to_scalar(&bu);
            prop_assert_eq!(scalar, to_scalar);
        }

        #[test]
        fn scalar_bigint256_roundtrip(scalar in scalar()) {
            // Shift right to make sure that the MSB is 0, so that the Scalar can be
            // converted to a BigInt256
            let shifted_scalar = scalar >> 1;
            let as_bigint256: BigInt256 = scalar_to_bigint256(shifted_scalar).unwrap();
            let to_scalar = bigint256_to_scalar(as_bigint256).unwrap();
            prop_assert_eq!(shifted_scalar, to_scalar);
        }
    }
}
