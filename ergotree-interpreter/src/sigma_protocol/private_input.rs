//! Private input types for the prover's secrets
use core::convert::TryInto;
use core::fmt::Formatter;

use alloc::vec::Vec;
use ergo_chain_types::ec_point::exponentiate_gen;
use ergo_chain_types::EcPoint;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;

use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

extern crate derive_more;
use derive_more::From;
use k256::elliptic_curve::PrimeField;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use super::wscalar::Wscalar;

/// Secret key of discrete logarithm signature protocol
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(from = "Wscalar", into = "Wscalar"))]
#[derive(PartialEq, Eq, Clone)]
pub struct DlogProverInput {
    /// secret key value
    pub w: Wscalar,
    pk: EcPoint,
}

impl core::fmt::Debug for DlogProverInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // to avoid leaking it in error messages, logs, etc.
        "DLOGPI:***".fmt(f)
    }
}

impl From<Wscalar> for DlogProverInput {
    fn from(scalar: Wscalar) -> Self {
        DlogProverInput::new(scalar)
    }
}

impl From<DlogProverInput> for Wscalar {
    fn from(prover_input: DlogProverInput) -> Self {
        prover_input.w
    }
}

impl DlogProverInput {
    /// Scalar(secret key) size in bytes
    pub const SIZE_BYTES: usize = 32;

    /// Create new DlogProverInput
    pub fn new(w: Wscalar) -> DlogProverInput {
        Self {
            pk: exponentiate_gen(w.as_scalar_ref()),
            w,
        }
    }
    /// generates random secret in the range [0, n), where n is DLog group order.
    #[cfg(feature = "std")]
    pub fn random() -> DlogProverInput {
        use ergotree_ir::sigma_protocol::dlog_group;

        use crate::sigma_protocol::crypto_utils;

        DlogProverInput::new(
            dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng()).into(),
        )
    }

    /// Attempts to parse the given byte array as an SEC-1-encoded scalar(secret key).
    /// Returns None if the byte array does not contain a big-endian integer in the range [0, modulus).
    pub fn from_bytes(bytes: &[u8; DlogProverInput::SIZE_BYTES]) -> Option<DlogProverInput> {
        k256::Scalar::from_repr((*bytes).into())
            .map(|s| DlogProverInput::new(Wscalar::from(s)))
            .into()
    }

    /// Attempts to parse the given Base16-encoded byte array as an SEC-1-encoded scalar(secret key).
    /// Returns None if the byte array does not contain a big-endian integer in the range [0, modulus).
    pub fn from_base16_str(str: &str) -> Option<DlogProverInput> {
        base16::decode(str)
            .ok()
            .and_then(|bytes| bytes.as_slice().try_into().ok().map(Self::from_bytes))
            .flatten()
    }

    /// Attempts to create DlogProverInput from BigUint
    /// Returns None if not in the range [0, modulus).
    pub fn from_biguint(b: BigUint) -> Option<DlogProverInput> {
        if b.bits() > (DlogProverInput::SIZE_BYTES * 8) as u64 {
            return None;
        }
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
        self.w.to_bytes()
    }

    /// public key of discrete logarithm signature protocol
    pub fn public_image(&self) -> ProveDlog {
        ProveDlog::new(self.pk)
    }

    /// Return true if the secret is 0
    pub fn is_zero(&self) -> bool {
        self.w.is_zero()
    }
}

/// Diffie-Hellman tuple and secret
/// Used in a proof that of equality of discrete logarithms (i.e., a proof of a Diffie-Hellman tuple):
/// given group elements g, h, u, v, the proof convinces a verifier that the prover knows `w` such
/// that `u = g^w` and `v = h^w`, without revealing `w`
#[derive(PartialEq, Eq, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct DhTupleProverInput {
    /// Diffie-Hellman tuple's secret
    #[cfg_attr(feature = "json", serde(rename = "secret"))]
    pub w: Wscalar,
    /// Diffie-Hellman tuple
    #[cfg_attr(feature = "json", serde(flatten))]
    pub common_input: ProveDhTuple,
}

impl core::fmt::Debug for DhTupleProverInput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // to avoid leaking it in error messages, logs, etc.
        "DHTPI:***".fmt(f)
    }
}

impl DhTupleProverInput {
    /// Size in bytes: 32(secret)+33(g)+33(h)+33(u)+33(v)=164 bytes
    pub const SIZE_BYTES: usize = DlogProverInput::SIZE_BYTES + EcPoint::GROUP_SIZE * 4;

    /// Create random secret and Diffie-Hellman tuple
    #[allow(clippy::many_single_char_names)]
    #[cfg(feature = "std")]
    pub fn random() -> DhTupleProverInput {
        use ergo_chain_types::ec_point::{exponentiate, generator};
        use ergotree_ir::sigma_protocol::dlog_group;
        let h = exponentiate_gen(&dlog_group::random_scalar_in_group_range(
            super::crypto_utils::secure_rng(),
        ));
        let w = dlog_group::random_scalar_in_group_range(super::crypto_utils::secure_rng());
        let u = exponentiate_gen(&w);
        let v = exponentiate(&h, &w);
        let common_input = ProveDhTuple::new(generator(), h, u, v);
        DhTupleProverInput {
            w: w.into(),
            common_input,
        }
    }

    /// Public image (Diffie-Hellman tuple)
    pub fn public_image(&self) -> &ProveDhTuple {
        &self.common_input
    }

    /// 32(secret)+33(g)+33(h)+33(u)+33(v)=164 bytes
    #[allow(clippy::unwrap_used)]
    pub fn to_bytes(&self) -> [u8; DhTupleProverInput::SIZE_BYTES] {
        let mut bytes = Vec::with_capacity(DhTupleProverInput::SIZE_BYTES);
        bytes.extend_from_slice(self.w.as_scalar_ref().to_bytes().as_slice());
        bytes.extend_from_slice(&self.common_input.g.sigma_serialize_bytes().unwrap());
        bytes.extend_from_slice(&self.common_input.h.sigma_serialize_bytes().unwrap());
        bytes.extend_from_slice(&self.common_input.u.sigma_serialize_bytes().unwrap());
        bytes.extend_from_slice(&self.common_input.v.sigma_serialize_bytes().unwrap());
        bytes.try_into().unwrap()
    }

    /// Parse from bytes (32(secret)+33(g)+33(h)+33(u)+33(v)=164 bytes)
    /// secret is expected as SEC-1-encoded scalar of 32 bytes,
    /// g,h,u,v are expected as 33-byte compressed points
    #[allow(clippy::unwrap_used)]
    pub fn from_bytes(bytes: &[u8; DhTupleProverInput::SIZE_BYTES]) -> Option<DhTupleProverInput> {
        let w_bytes: &[u8; DlogProverInput::SIZE_BYTES] =
            &bytes[..DlogProverInput::SIZE_BYTES].try_into().unwrap();
        let g_bytes: &[u8; EcPoint::GROUP_SIZE] = &bytes
            [DlogProverInput::SIZE_BYTES..DlogProverInput::SIZE_BYTES + EcPoint::GROUP_SIZE]
            .try_into()
            .unwrap();
        let h_bytes: &[u8; EcPoint::GROUP_SIZE] = &bytes[DlogProverInput::SIZE_BYTES
            + EcPoint::GROUP_SIZE
            ..DlogProverInput::SIZE_BYTES + EcPoint::GROUP_SIZE * 2]
            .try_into()
            .unwrap();
        let u_bytes: &[u8; EcPoint::GROUP_SIZE] = &bytes[DlogProverInput::SIZE_BYTES
            + EcPoint::GROUP_SIZE * 2
            ..DlogProverInput::SIZE_BYTES + EcPoint::GROUP_SIZE * 3]
            .try_into()
            .unwrap();
        let v_bytes: &[u8; EcPoint::GROUP_SIZE] = &bytes[DlogProverInput::SIZE_BYTES
            + EcPoint::GROUP_SIZE * 3
            ..DlogProverInput::SIZE_BYTES + EcPoint::GROUP_SIZE * 4]
            .try_into()
            .unwrap();
        Self::from_bytes_fields(w_bytes, g_bytes, h_bytes, u_bytes, v_bytes)
    }

    /// Parse from bytes
    /// secret is expected as SEC-1-encoded scalar of 32 bytes,
    /// g,h,u,v are expected as 33-byte compressed points
    pub fn from_bytes_fields(
        w_bytes: &[u8; DlogProverInput::SIZE_BYTES],
        g_bytes: &[u8; EcPoint::GROUP_SIZE],
        h_bytes: &[u8; EcPoint::GROUP_SIZE],
        u_bytes: &[u8; EcPoint::GROUP_SIZE],
        v_bytes: &[u8; EcPoint::GROUP_SIZE],
    ) -> Option<DhTupleProverInput> {
        let w: Option<Wscalar> = k256::Scalar::from_repr((*w_bytes).into())
            .map(Wscalar::from)
            .into();
        let g = EcPoint::sigma_parse_bytes(&g_bytes[..EcPoint::GROUP_SIZE]).ok()?;
        let h = EcPoint::sigma_parse_bytes(&h_bytes[..EcPoint::GROUP_SIZE]).ok()?;
        let u = EcPoint::sigma_parse_bytes(&u_bytes[..EcPoint::GROUP_SIZE]).ok()?;
        let v = EcPoint::sigma_parse_bytes(&v_bytes[..EcPoint::GROUP_SIZE]).ok()?;
        w.map(|w| DhTupleProverInput {
            w,
            common_input: ProveDhTuple::new(g, h, u, v),
        })
    }
}

/// Private inputs (secrets)
#[derive(PartialEq, Eq, Debug, Clone, From)]
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn from_biguint_rejects_values_ge_modulus_and_2_pow_256() {
        // secp256k1 group order n = FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
        let n = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
            16,
        )
        .unwrap();
        let one = BigUint::from(1u32);
        let two_pow_256: BigUint = BigUint::from(1u32) << 256u32;

        // Valid range [0, n)
        assert!(DlogProverInput::from_biguint(BigUint::from(0u32)).is_some());
        assert!(DlogProverInput::from_biguint(one.clone()).is_some());
        assert!(DlogProverInput::from_biguint(&n - &one).is_some());

        // Out of range [n, 2^256)
        assert!(DlogProverInput::from_biguint(n.clone()).is_none());
        assert!(DlogProverInput::from_biguint(&n + &one).is_none());

        // >= 2^256 previously silently truncated to low 256 bits (turning 2^256 into 0)
        assert!(DlogProverInput::from_biguint(two_pow_256.clone()).is_none());
        assert!(DlogProverInput::from_biguint(&two_pow_256 + &one).is_none());
        assert!(DlogProverInput::from_biguint(&two_pow_256 + BigUint::from(42u32)).is_none());
    }
}
