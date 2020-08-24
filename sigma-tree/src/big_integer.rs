// use k256::Scalar;
use num_bigint::BigInt;
// use std::convert::TryFrom;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BigInteger(BigInt);

impl From<BigInt> for BigInteger {
    fn from(b: BigInt) -> Self {
        BigInteger(b)
    }
}
