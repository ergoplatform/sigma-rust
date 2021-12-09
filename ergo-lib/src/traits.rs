/// Get the length of constant sized arrays.
///
/// ```
/// type SecretKeyBytes = [u8; 32];
/// let secret_key_length = SecretKeyBytes::LEN; // 32
/// let copy_buf = [0u8; SecretKeyBytes::LEN];
/// ```
pub trait ArrLength {
    /// Length of the array
    const LEN: usize;
}

impl<T, const LENGTH: usize> ArrLength for [T; LENGTH] {
    const LEN: usize = LENGTH;
}
