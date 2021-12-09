/// Get the length of constant sized arrays.
///
/// ```
/// use ergo_lib::ArrLength;
///
/// type SecretKeyBytes = [u8; 32];
///
/// assert_eq!(32, SecretKeyBytes::LEN)
/// ```
pub trait ArrLength {
    /// Length of the array
    const LEN: usize;
}

impl<T, const LENGTH: usize> ArrLength for [T; LENGTH] {
    const LEN: usize = LENGTH;
}
