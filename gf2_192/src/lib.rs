pub mod gf2_192;

/// Logical right shift of i64 values which is equivalent to the `>>>` operator in java/scala. Need
/// this because rust does 'arithmetical shift right' on signed integers.
/// See:
///  - https://en.wikipedia.org/wiki/Logical_shift
///  - https://en.wikipedia.org/wiki/Arithmetic_shift
pub fn lrs(b: i64, s: i64) -> i64 {
    ((b as u64) >> s) as i64
}
