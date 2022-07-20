//! Private input types for the prover's secrets
use std::convert::TryInto;
use std::fmt::Formatter;

use elliptic_curve::group::ff::PrimeField;
use ergotree_ir::sigma_protocol::dlog_group;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;

use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

extern crate derive_more;
use derive_more::From;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use super::crypto_utils;
use super::wscalar::Wscalar;

/// Secret key of discrete logarithm signature protocol
#[derive(PartialEq, Clone, derive_more::From)]
pub struct DlogProverInput {
    /// secret key value
    pub w: Wscalar,
}

impl std::fmt::Debug for DlogProverInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        "DLOGPI:***".fmt(f)
    }
}

impl DlogProverInput {
    /// Scalar(secret key) size in bytes
    pub const SIZE_BYTES: usize = 32;

    /// generates random secret in the range [0, n), where n is DLog group order.
    pub fn random() -> DlogProverInput {
        DlogProverInput {
            w: dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng()).into(),
        }
    }

    /// Attempts to parse the given byte array as an SEC-1-encoded scalar(secret key).
    /// Returns None if the byte array does not contain a big-endian integer in the range [0, modulus).
    pub fn from_bytes(bytes: &[u8; DlogProverInput::SIZE_BYTES]) -> Option<DlogProverInput> {
        k256::Scalar::from_repr((*bytes).into())
            .map(|s| DlogProverInput::from(Wscalar::from(s)))
            .into()
    }

    /// Attempts to parse the given Base16-encoded byte array as an SEC-1-encoded scalar(secret key).
    /// Returns None if the byte array does not contain a big-endian integer in the range [0, modulus).
    pub fn from_base16_str(str: String) -> Option<DlogProverInput> {
        base16::decode(&str)
            .ok()
            .and_then(|bytes| bytes.as_slice().try_into().ok().map(Self::from_bytes))
            .flatten()
    }

    /// Attempts to create DlogProverInput from BigUint
    /// Returns None if not in the range [0, modulus).
    pub fn from_biguint(b: BigUint) -> Option<DlogProverInput> {
        /// Converts a BigUint to a byte array (big-endian).
        #[allow(clippy::unwrap_used)]
        pub fn biguint_to_32bytes(x: &BigUint) -> [u8; 32] {
            let mask = BigUint::from(u8::MAX);
            let mut bytes = [0u8; 32];
            (0..32).for_each(|i| {
                bytes[i] = ((x >> ((31 - i) * 8)) as BigUint & &mask).to_u8().unwrap();
            });
            bytes
        }
        let bytes = biguint_to_32bytes(&b);
        Self::from_bytes(&bytes)
    }

    /// byte representation of the underlying scalar
    pub fn to_bytes(&self) -> [u8; DlogProverInput::SIZE_BYTES] {
        self.w.as_scalar_ref().to_bytes().into()
    }

    /// public key of discrete logarithm signature protocol
    pub fn public_image(&self) -> ProveDlog {
        // test it, see https://github.com/ergoplatform/sigma-rust/issues/38
        let g = ergo_chain_types::ec_point::generator();
        ProveDlog::new(ergo_chain_types::ec_point::exponentiate(
            &g,
            self.w.as_scalar_ref(),
        ))
    }

    /// Return true if the secret is 0
    pub fn is_zero(&self) -> bool {
        self.w.as_scalar_ref().is_zero().into()
    }
}

/// Diffie-Hellman tuple and secret
/// Used in a proof that of equality of discrete logarithms (i.e., a proof of a Diffie-Hellman tuple):
/// given group elements g, h, u, v, the proof convinces a verifier that the prover knows `w` such
/// that `u = g^w` and `v = h^w`, without revealing `w`
#[derive(PartialEq, Clone)]
pub struct DhTupleProverInput {
    /// Diffie-Hellman tuple's secret
    pub w: Wscalar,
    /// Diffie-Hellman tuple
    pub common_input: ProveDhTuple,
}

impl std::fmt::Debug for DhTupleProverInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "DHTPI:***".fmt(f)
    }
}

impl DhTupleProverInput {
    /// Create random secret and Diffie-Hellman tuple
    #[allow(clippy::many_single_char_names)]
    pub fn random() -> DhTupleProverInput {
        use ergo_chain_types::ec_point::{exponentiate, generator};
        let g = generator();
        let h = exponentiate(
            &generator(),
            &dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng()),
        );
        let w = dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng());
        let u = exponentiate(&g, &w);
        let v = exponentiate(&h, &w);
        let common_input = ProveDhTuple::new(g, h, u, v);
        DhTupleProverInput {
            w: w.into(),
            common_input,
        }
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
