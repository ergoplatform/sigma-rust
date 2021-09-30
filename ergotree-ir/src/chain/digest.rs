//! Main "remote" type for [Digest]()

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SigmaParsingError,
    SigmaSerializable, SigmaSerializeResult,
};
use crate::util::AsVecI8;

/// Blake2b256 hash (256 bit)
pub fn blake2b256_hash(bytes: &[u8]) -> Digest32 {
    Digest(sigma_util::hash::blake2b256_hash(bytes))
}

/// N-bytes array in a box. `Digest32` is most type synonym.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Digest<const N: usize>(pub Box<[u8; N]>);

/// 32 byte array used as ID of some value: block, transaction, etc.
/// Usually this is as blake2b hash of serialized form
pub type Digest32 = Digest<32>;

/// AVL tree digest: root hash along with tree height (33 bytes)
pub type ADDigest = Digest<33>;

impl<const N: usize> Digest<N> {
    /// Digest size bytes
    pub const SIZE: usize = N;

    /// All zeros
    pub fn zero() -> Digest<N> {
        Digest(Box::new([0u8; N]))
    }
}

impl<const N: usize> Default for Digest<N> {
    fn default() -> Self {
        Digest::zero()
    }
}

impl<const N: usize> From<[u8; N]> for Digest<N> {
    fn from(bytes: [u8; N]) -> Self {
        Digest(Box::new(bytes))
    }
}

impl<const N: usize> From<Digest<N>> for [u8; N] {
    fn from(v: Digest<N>) -> Self {
        *v.0
    }
}

impl<const N: usize> From<Digest<N>> for Vec<u8> {
    fn from(v: Digest<N>) -> Self {
        v.0.to_vec()
    }
}

impl<const N: usize> From<Digest<N>> for Vec<i8> {
    fn from(v: Digest<N>) -> Self {
        v.0.to_vec().as_vec_i8()
    }
}

impl AsRef<[u8]> for Digest32 {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl<const N: usize> SigmaSerializable for Digest<N> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.write_all(self.0.as_ref())?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let mut bytes = [0; N];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes.into()))
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    #![allow(clippy::expect_used)]

    use std::convert::TryInto;

    use proptest::prelude::{Arbitrary, BoxedStrategy};
    use proptest::{collection::vec, prelude::*};

    use super::Digest;

    impl<const N: usize> Arbitrary for Digest<N> {
        type Parameters = ();
        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            vec(any::<u8>(), Self::SIZE)
                .prop_map(|v| {
                    Digest(Box::new(
                        v.try_into()
                            .expect("internal error: vec has length != Digest::SIZE"),
                    ))
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}
