pub mod gf2_192;

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
