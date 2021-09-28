//! Digest type used to represent hash functions output bytes.

/// N-bytes array in a box. `Digest32` is most type synonym.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Digest<const N: usize>(pub Box<[u8; N]>);
