//! Implementation of finite field arithmetic and polynomial interpolation/evaluation in Galois
//! field GF(2^192).

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]
// Clippy exclusions
#![allow(clippy::unit_arg)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use derive_more::From;
use gf2_192poly::Gf2_192PolyError;
use thiserror::Error;

pub mod gf2_192;
pub mod gf2_192poly;

/// Logical right shift of i64 values which is equivalent to the `>>>` operator in java/scala. Need
/// this because rust does 'arithmetical shift right' on signed integers.
/// See:
///  - <https://en.wikipedia.org/wiki/Logical_shift>
///  - <https://en.wikipedia.org/wiki/Arithmetic_shift>
pub fn lrs_i64(b: i64, s: i64) -> i64 {
    ((b as u64) >> s) as i64
}

/// Logical right shift of i8 values which is equivalent to the `>>>` operator in java/scala. Need
/// this because rust does 'arithmetical shift right' on signed integers.
/// See:
///  - <https://en.wikipedia.org/wiki/Logical_shift>
///  - <https://en.wikipedia.org/wiki/Arithmetic_shift>
pub fn lrs_i8(b: i8, s: i8) -> i8 {
    ((b as u8) >> s) as i8
}

/// General error type for the `gf2_192` crate
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum Gf2_192Error {
    /// Failed to create `Gf2_192` from `&[i8]`
    #[error("Failed to create `Gf2_192` from `&[i8]`")]
    Gf2_192TryFromByteArrayError,
    /// Failed to create `Gf2_192` from `&[i8]`
    #[error("Failed to write `Gf2_192` to `&[i8]`")]
    Gf2_192ToByteArrayError,
    /// `Gf2_192Poly` error
    #[error("`Gf2_192Poly` Error: {0}")]
    Gf2_192PolyError(Gf2_192PolyError),
}
