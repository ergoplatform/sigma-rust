//! Private input types for the prover's secrets
use std::convert::TryInto;

use elliptic_curve::group::ff::PrimeField;
use ergotree_ir::sigma_protocol::dlog_group;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;

use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use k256::Scalar;

extern crate derive_more;
use derive_more::From;
use num_bigint::BigUint;

use super::crypto_utils;

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
            w: dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng()),
        }
    }

    /// Attempts to parse the given byte array as an SEC-1-encoded scalar(secret key).
    /// Returns None if the byte array does not contain a big-endian integer in the range [0, modulus).
    pub fn from_bytes(bytes: &[u8; DlogProverInput::SIZE_BYTES]) -> Option<DlogProverInput> {
        Scalar::from_repr((*bytes).into()).map(DlogProverInput::from)
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

/// Diffie-Hellman tuple and secret
/// Used in a proof that of equality of discrete logarithms (i.e., a proof of a Diffie-Hellman tuple):
/// given group elements g, h, u, v, the proof convinces a verifier that the prover knows `w` such
/// that `u = g^w` and `v = h^w`, without revealing `w`
#[derive(PartialEq, Debug, Clone)]
pub struct DhTupleProverInput {
    /// Diffie-Hellman tuple's secret
    pub w: Scalar,
    /// Diffie-Hellman tuple
    pub common_input: ProveDhTuple,
}

impl DhTupleProverInput {
    /// Create random secret and Diffie-Hellman tuple
    #[allow(clippy::many_single_char_names)]
    pub fn random() -> DhTupleProverInput {
        let g = dlog_group::generator();
        let h = dlog_group::exponentiate(
            &dlog_group::generator(),
            &dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng()),
        );
        let w = dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng());
        let u = dlog_group::exponentiate(&g, &w);
        let v = dlog_group::exponentiate(&h, &w);
        let common_input = ProveDhTuple::new(g, h, u, v);
        DhTupleProverInput { w, common_input }
    }

    /// Public image (Diffie-Hellman tuple)
    pub fn public_image(&self) -> &ProveDhTuple {
        &self.common_input
    }
}

/// Private inputs (secrets)
#[derive(PartialEq, Debug, Clone, From)]
pub enum PrivateInput {
    /// Discrete logarithm prover input
    DlogProverInput(DlogProverInput),
    /// Diffie-Hellman tuple prover input
    DhTupleProverInput(DhTupleProverInput),
}

impl PrivateInput {
    /// Public image of the private input
    pub fn public_image(&self) -> SigmaBoolean {
        match self {
            PrivateInput::DlogProverInput(dl) => dl.public_image().into(),
            PrivateInput::DhTupleProverInput(dht) => dht.public_image().clone().into(),
        }
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
pub(crate) mod arbitrary {

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for DlogProverInput {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(DlogProverInput::random()),
                Just(DlogProverInput::random()),
                Just(DlogProverInput::random()),
                Just(DlogProverInput::random()),
                Just(DlogProverInput::random()),
            ]
            .boxed()
        }
    }

    impl Arbitrary for DhTupleProverInput {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(DhTupleProverInput::random()),
                Just(DhTupleProverInput::random()),
                Just(DhTupleProverInput::random()),
                Just(DhTupleProverInput::random()),
                Just(DhTupleProverInput::random()),
            ]
            .boxed()
        }
    }

    impl Arbitrary for PrivateInput {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                any::<DlogProverInput>().prop_map_into(),
                any::<DhTupleProverInput>().prop_map_into(),
            ]
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {}
