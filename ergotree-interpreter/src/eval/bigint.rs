use lazy_static::lazy_static;
use num_bigint::BigInt;
use num_traits::pow::Pow;

lazy_static! {
    pub static ref MAX_BOUND: BigInt = Pow::pow(BigInt::from(2), 255u32) - 1;
    pub static ref MIN_BOUND: BigInt = -Pow::pow(BigInt::from(2), 255u32);
}

pub fn fits_in_256_bits(b: &BigInt) -> bool {
    *b >= *MIN_BOUND && *b <= *MAX_BOUND
}
