use k256::arithmetic::Scalar;
use num_bigint::BigInt;
// use std::convert::TryFrom;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BigInteger(BigInt);

impl From<BigInt> for BigInteger {
    fn from(b: BigInt) -> Self {
        BigInteger(b)
    }
}

impl From<Scalar> for BigInteger {
    fn from(s: Scalar) -> Self {
        let bytes = s.to_bytes();
        BigInt::from_signed_bytes_be(&bytes[..]).into()
    }
}

// impl TryFrom<BigInteger> for Scalar {
//     type Error;
//     fn try_from(value: BigInteger) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }
//

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma_protocol::dlog_group;
    use std::convert::TryInto;

    #[test]
    fn scalar_conversion_roundtrip() {
        let s = dlog_group::random_scalar_in_group_range();
        let b: BigInteger = s.into();
        let bytes = b.0.to_signed_bytes_be();
        let s2 =
            Scalar::from_bytes(bytes.as_slice().try_into().expect("expected 32 bytes")).unwrap();
        // TODO: failed on CI with non-32 bytes length
        let b2: BigInteger = s2.into();
        assert_eq!(b, b2);
    }
}
