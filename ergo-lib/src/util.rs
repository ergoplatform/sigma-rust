//! Utilities

use elliptic_curve::subtle::CtOption;

/// Convert to Option<T>
pub(crate) trait IntoOption<T> {
    /// Get Option<T>
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOption<T> for CtOption<T> {
    fn into_option(self) -> Option<T> {
        if self.is_some().into() {
            Some(self.unwrap())
        } else {
            None
        }
    }
}

/// Vec<i8> to Vec<u8> conversion
pub trait FromVecI8 {
    /// Convert Vec<i8> to Vec<u8>
    fn from_vec_i8(bs: Vec<i8>) -> Self;
}

impl FromVecI8 for Vec<u8> {
    fn from_vec_i8(bs: Vec<i8>) -> Self {
        bs.iter().map(|b| *b as u8).collect()
    }
}

/// Convert Vec<i8> to Vec<u8>
pub trait AsVecU8 {
    /// Returns as Vec<u8>
    fn as_vec_u8(self) -> Vec<u8>;
}

impl AsVecU8 for Vec<i8> {
    fn as_vec_u8(self) -> Vec<u8> {
        Vec::<u8>::from_vec_i8(self)
    }
}
