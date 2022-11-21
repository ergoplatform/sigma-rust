//! Hash functions

/// Blake2b256 hash (256 bit)
pub fn blake2b256_hash(bytes: &[u8]) -> Box<[u8; 32]> {
    use blake2::digest::typenum::U32;
    use blake2::Blake2b;
    use blake2::Digest;

    type Blake2b256 = Blake2b<U32>;

    let mut hasher = Blake2b256::new();
    hasher.update(bytes);
    let hash: [u8; 32] = hasher.finalize().into();
    Box::new(hash)
}

/// Sha256 hash (256 bit)
pub fn sha256_hash(bytes: &[u8]) -> Box<[u8; 32]> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Box::new(*hasher.finalize().as_ref())
}
