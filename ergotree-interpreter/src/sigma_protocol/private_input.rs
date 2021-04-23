//! Private input types for the prover's secrets
use std::convert::TryInto;

use ergotree_ir::sigma_protocol::dlog_group;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;

use k256::elliptic_curve::ff::PrimeField;
use k256::Scalar;

extern crate derive_more;
use derive_more::From;
use num_bigint::BigUint;

/// Secret key of discrete logarithm signature protocol
#[derive(PartialEq, Debug, Clone)]
pub struct DlogProverInput {
    /// secret key value
    pub w: Scalar,
}

impl DlogProverInput {
    /// Scalar(secret key) size in bytes
    pub const SIZE_BYTES: usize = 32;

    /// generates random secret in the range [0, n), where n is DLog group order.
    pub fn random() -> DlogProverInput {
        DlogProverInput {
            w: dlog_group::random_scalar_in_group_range(),
        }
    }

    /// Attempts to parse the given byte array as an SEC-1-encoded scalar(secret key).
    /// Returns None if the byte array does not contain a big-endian integer in the range [0, modulus).
    pub fn from_bytes(bytes: &[u8; DlogProverInput::SIZE_BYTES]) -> Option<DlogProverInput> {
        Scalar::from_repr(bytes.clone().into()).map(DlogProverInput::from)
    }

    /// Attempts to parse the given Base16-encoded byte array as an SEC-1-encoded scalar(secret key).
    /// Returns None if the byte array does not contain a big-endian integer in the range [0, modulus).
    pub fn from_base16_str(str: String) -> Option<DlogProverInput> {
        base16::decode(&str)
            .ok()
            .map(|bytes| bytes.as_slice().try_into().ok().map(Self::from_bytes))
            .flatten()
            .flatten()
    }

    /// Attempts to create DlogProverInput from BigUint
    /// Returns None if not in the range [0, modulus).
    pub fn from_biguint(b: BigUint) -> Option<DlogProverInput> {
        let bytes = b.to_bytes_be();
        bytes
            .as_slice()
            .try_into()
            .ok()
            .map(Self::from_bytes)
            .flatten()
    }

    /// byte representation of the underlying scalar
    pub fn to_bytes(&self) -> [u8; DlogProverInput::SIZE_BYTES] {
        self.w.to_bytes().into()
    }

    /// public key of discrete logarithm signature protocol
    pub fn public_image(&self) -> ProveDlog {
        // test it, see https://github.com/ergoplatform/sigma-rust/issues/38
        let g = dlog_group::generator();
        ProveDlog::new(dlog_group::exponentiate(&g, &self.w))
    }
}

impl From<Scalar> for DlogProverInput {
    fn from(w: Scalar) -> Self {
        DlogProverInput { w }
    }
}

/// Private inputs (secrets)
#[derive(PartialEq, Debug, Clone, From)]
pub enum PrivateInput {
    /// Discrete logarithm prover input
    DlogProverInput(DlogProverInput),
    /// DH tuple prover input
    DiffieHellmanTupleProverInput,
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for DlogProverInput {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![Just(DlogProverInput::random()),].boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {}
