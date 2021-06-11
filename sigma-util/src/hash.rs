//! Hash functions

use std::convert::TryInto;

/// Blake2b256 hash (256 bit)
pub fn blake2b256_hash(bytes: &[u8]) -> Box<[u8; 32]> {
    use blake2::digest::{Update, VariableOutput};
    use blake2::VarBlake2b;

    // unwrap is safe 32 bytes is a valid hash size (<= 512 && 32 % 8 == 0)
    let mut hasher = VarBlake2b::new(32).unwrap();
    hasher.update(bytes);
    let hash = hasher.finalize_boxed();
    // unwrap is safe due to hash size is expected to be 32
    hash.try_into().unwrap()
}

/// Sha256 hash (256 bit)
pub fn sha256_hash(bytes: &[u8]) -> Box<[u8; 32]> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Box::new(*hasher.finalize().as_ref())
}
