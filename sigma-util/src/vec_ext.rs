//! Vec extensions

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
    /// Returns Vec<u8>
    fn as_vec_u8(&self) -> Vec<u8>;
}

impl AsVecU8 for Vec<i8> {
    fn as_vec_u8(&self) -> Vec<u8> {
        Vec::<u8>::from_vec_i8(self.clone())
    }
}

/// Convert Vec<u8> to Vec<i8>
pub trait AsVecI8 {
    /// Returns Vec<i8>
    fn as_vec_i8(&self) -> Vec<i8>;
}

impl AsVecI8 for Vec<u8> {
    fn as_vec_i8(&self) -> Vec<i8> {
        self.iter().map(|b| *b as i8).collect()
    }
}
