//! Discrete logarithm signature protocol

use super::wscalar::Wscalar;
use super::ProverMessage;
use alloc::boxed::Box;
use alloc::vec::Vec;
use ergo_chain_types::EcPoint;
use ergotree_ir::serialization::SigmaSerializable;

/// First message from the prover (message `a` of `SigmaProtocol`) for discrete logarithm case
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct FirstDlogProverMessage {
    #[cfg_attr(feature = "json", serde(rename = "a"))]
    pub(crate) a: Box<EcPoint>,
}

impl FirstDlogProverMessage {
    /// `a` of `SigmaProtocol`
    pub fn a(&self) -> &EcPoint {
        &self.a
    }
}

impl From<EcPoint> for FirstDlogProverMessage {
    fn from(ecp: EcPoint) -> Self {
        FirstDlogProverMessage { a: ecp.into() }
    }
}

impl ProverMessage for FirstDlogProverMessage {
    fn bytes(&self) -> Vec<u8> {
        #[allow(clippy::unwrap_used)]
        // EcPoint serialization can only on OOM
        self.a.sigma_serialize_bytes().unwrap()
    }
}

/// Second message from the prover (message `z` of `SigmaProtocol`) for discrete logarithm case
#[derive(PartialEq, Eq, Debug, Clone, derive_more::From, derive_more::Into)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct SecondDlogProverMessage {
    /// message `z`
    pub z: Wscalar,
}

/// Interactive prover
pub mod interactive_prover {
    use alloc::boxed::Box;
    use core::ops::Mul;

    use super::{FirstDlogProverMessage, SecondDlogProverMessage};
    use crate::sigma_protocol::wscalar::Wscalar;
    use crate::sigma_protocol::{private_input::DlogProverInput, Challenge};
    use blake2::Blake2b;
    use blake2::Digest;
    use ergo_chain_types::ec_point::exponentiate_gen;
    use ergo_chain_types::{
        ec_point::{exponentiate, inverse},
        EcPoint,
    };
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
    use k256::elliptic_curve::ops::Reduce;
    use k256::Scalar;

    /// Step 5 from <https://ergoplatform.org/docs/ErgoScript.pdf>
    /// For every leaf marked “simulated”, use the simulator of the sigma protocol for that leaf
    /// to compute the commitment "a" and the response "z", given the challenge "e" that
    /// is already stored in the leaf
    #[cfg(feature = "std")]
    pub(crate) fn simulate(
        public_input: &ProveDlog,
        challenge: &Challenge,
    ) -> (FirstDlogProverMessage, SecondDlogProverMessage) {
        use ergotree_ir::sigma_protocol::dlog_group;
        //SAMPLE a random z <- Zq
        let z = dlog_group::random_scalar_in_group_range(
            crate::sigma_protocol::crypto_utils::secure_rng(),
        );

        //COMPUTE a = g^z*h^(-e)  (where -e here means -e mod q)
        let e: Scalar = challenge.into();
        let minus_e = e.negate();
        let h_to_e = exponentiate(&public_input.h, &minus_e);
        let g_to_z = exponentiate_gen(&z);
        let a = g_to_z * &h_to_e;
        (
            FirstDlogProverMessage { a: a.into() },
            SecondDlogProverMessage { z: z.into() },
        )
    }

    /// Step 6 from <https://ergoplatform.org/docs/ErgoScript.pdf>
    /// For every leaf marked “real”, use the first prover step of the sigma protocol for
    /// that leaf to compute the necessary randomness "r" and the commitment "a"
    #[cfg(feature = "std")]
    pub fn first_message() -> (Wscalar, FirstDlogProverMessage) {
        use ergotree_ir::sigma_protocol::dlog_group;

        use crate::sigma_protocol::crypto_utils;
        let r = dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng());
        let a = exponentiate_gen(&r);
        (r.into(), FirstDlogProverMessage { a: a.into() })
    }

    /// Step 6 from <https://ergoplatform.org/docs/ErgoScript.pdf>
    /// Generate first message "nonce" deterministically, optionally using auxilliary rng
    /// # Safety
    /// This is only intended to be used in single-signer scenarios.
    /// Using this in multi-signature situations where other (untrusted) signers influence the signature can cause private key leakage by producing multiple signatures for the same message with the same nonce
    pub fn first_message_deterministic(
        sk: &DlogProverInput,
        msg: &[u8],
        aux_rand: &[u8],
    ) -> (Wscalar, FirstDlogProverMessage) {
        // This is based on BIP340 deterministic nonces, see: https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki#default-signing
        type Blake2b256 = Blake2b<blake2::digest::typenum::U32>;
        const AUX_TAG: &[u8] = b"erg/aux";
        // Perform domain seperation so alternative signature schemes don't end up producing the same nonce, for example ProveDHTuple with deterministic nonces
        const NONCE_TAG: &[u8] = b"ergprovedlog/nonce";

        let aux_rand_hash: [u8; 32] = Blake2b256::new()
            .chain_update(AUX_TAG)
            .chain_update(aux_rand)
            .finalize()
            .into();
        let mut sk_bytes = sk.w.as_scalar_ref().to_bytes();
        sk_bytes
            .iter_mut()
            .zip(aux_rand_hash)
            .for_each(|(a, b)| *a ^= b);
        #[allow(clippy::unwrap_used)] // unwrap will only fail if OOM
        let hash: [u8; 32] = Blake2b256::new()
            .chain_update(NONCE_TAG)
            .chain_update(sk_bytes)
            .chain_update(sk.public_image().h.sigma_serialize_bytes().unwrap())
            .chain_update(msg)
            .finalize()
            .into();

        let r = <Scalar as Reduce<k256::FieldBytes>>::reduce(&hash.into());
        (
            r.into(),
            FirstDlogProverMessage {
                a: Box::new(exponentiate_gen(&r)),
            },
        )
    }

    /// Step 9 part 2 from <https://ergoplatform.org/docs/ErgoScript.pdf>
    /// compute its response "z" according to the second prover step(step 5 in whitepaper)
    /// of the sigma protocol given the randomness "r"(rnd) used for the commitment "a",
    /// the challenge "e", and witness w.
    pub(crate) fn second_message(
        private_input: &DlogProverInput,
        rnd: Wscalar,
        challenge: &Challenge,
    ) -> SecondDlogProverMessage {
        let e: Scalar = challenge.into();
        // modulo multiplication, no need to explicit mod op
        let ew = e.mul(private_input.w.as_scalar_ref());
        // modulo addition, no need to explicit mod op
        let z = rnd.as_scalar_ref().add(&ew);
        SecondDlogProverMessage { z: z.into() }
    }

    /// The function computes initial prover's commitment to randomness
    /// ("a" message of the sigma-protocol) based on the verifier's challenge ("e")
    /// and prover's response ("z")
    ///
    /// g^z = a*h^e => a = g^z/h^e
    pub fn compute_commitment(
        proposition: &ProveDlog,
        challenge: &Challenge,
        second_message: &SecondDlogProverMessage,
    ) -> EcPoint {
        let h = *proposition.h;
        let e: Scalar = challenge.into();
        let g_z = exponentiate_gen(second_message.z.as_scalar_ref());
        let h_e = exponentiate(&h, &e);
        g_z * &inverse(&h_e)
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::super::*;
    use super::*;
    use crate::sigma_protocol::private_input::DlogProverInput;

    use fiat_shamir::fiat_shamir_hash_fn;
    use proptest::collection::vec;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_compute_commitment(secret in any::<DlogProverInput>(), challenge in any::<Challenge>()) {
            let pk = secret.public_image();
            let (r, commitment) = interactive_prover::first_message();
            let second_message = interactive_prover::second_message(&secret, r, &challenge);
            let a = interactive_prover::compute_commitment(&pk, &challenge, &second_message);
            prop_assert_eq!(a, *commitment.a);
        }

        #[test]
        fn test_deterministic_commitment(secret in any::<DlogProverInput>(), secret2 in any::<DlogProverInput>(), message in vec(any::<u8>(), 0..100000)) {
            fn sign(secret: &DlogProverInput, message: &[u8]) -> EcPoint {
                let pk = secret.public_image();
                let challenge: Challenge = fiat_shamir_hash_fn(message).into();
                let (r, _) = interactive_prover::first_message_deterministic(secret, message, &[]);
                let second_message = interactive_prover::second_message(secret, r, &challenge);
                interactive_prover::compute_commitment(&pk, &challenge, &second_message)
            }
            let a = sign(&secret, &message);
            let a2 = sign(&secret2, &message);
            prop_assert_ne!(a, a2);
        }
    }
}
