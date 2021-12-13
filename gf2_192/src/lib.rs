use derive_more::From;
use gf2_192poly::Gf2_192PolyError;
use thiserror::Error;

pub mod gf2_192;
pub mod gf2_192poly;

/// Logical right shift of i64 values which is equivalent to the `>>>` operator in java/scala. Need
/// this because rust does 'arithmetical shift right' on signed integers.
/// See:
///  - https://en.wikipedia.org/wiki/Logical_shift
///  - https://en.wikipedia.org/wiki/Arithmetic_shift
pub fn lrs_i64(b: i64, s: i64) -> i64 {
    ((b as u64) >> s) as i64
}

/// Logical right shift of i8 values which is equivalent to the `>>>` operator in java/scala. Need
/// this because rust does 'arithmetical shift right' on signed integers.
/// See:
///  - https://en.wikipedia.org/wiki/Logical_shift
///  - https://en.wikipedia.org/wiki/Arithmetic_shift
pub fn lrs_i8(b: i8, s: i8) -> i8 {
    ((b as u8) >> s) as i8
}

/// General error type for the `gf2_192` crate
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum Gf2_192Error {
    /// Failed to create `Gf2_192` from `&[i8]`
    #[error("Failed to create `Gf2_192` from `&[i8]`")]
    Gf2_192TryFromByteArrayError,
    /// `Gf2_192Poly` error
    #[error("`Gf2_192Poly` Error: {0}")]
    Gf2_192PolyError(Gf2_192PolyError),
}
