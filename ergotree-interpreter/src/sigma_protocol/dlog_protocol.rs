//! Discrete logarithm signature protocol

use super::wscalar::Wscalar;
use super::ProverMessage;
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
    use std::ops::Mul;

    use super::{FirstDlogProverMessage, SecondDlogProverMessage};
    use crate::sigma_protocol::crypto_utils;
    use crate::sigma_protocol::wscalar::Wscalar;
    use crate::sigma_protocol::{private_input::DlogProverInput, Challenge};
    use ergo_chain_types::{
        ec_point::{exponentiate, generator, inverse},
        EcPoint,
    };
    use ergotree_ir::sigma_protocol::dlog_group;
    use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
    use k256::Scalar;

    /// Step 5 from <https://ergoplatform.org/docs/ErgoScript.pdf>
    /// For every leaf marked “simulated”, use the simulator of the sigma protocol for that leaf
    /// to compute the commitment "a" and the response "z", given the challenge "e" that
    /// is already stored in the leaf
    pub(crate) fn simulate(
        public_input: &ProveDlog,
        challenge: &Challenge,
    ) -> (FirstDlogProverMessage, SecondDlogProverMessage) {
        //SAMPLE a random z <- Zq
        let z = dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng());

        //COMPUTE a = g^z*h^(-e)  (where -e here means -e mod q)
        let e: Scalar = challenge.clone().into();
        let minus_e = e.negate();
        let h_to_e = exponentiate(&public_input.h, &minus_e);
        let g_to_z = exponentiate(&generator(), &z);
        let a = g_to_z * &h_to_e;
        (
            FirstDlogProverMessage { a: a.into() },
            SecondDlogProverMessage { z: z.into() },
        )
    }

    /// Step 6 from <https://ergoplatform.org/docs/ErgoScript.pdf>
    /// For every leaf marked “real”, use the first prover step of the sigma protocol for
    /// that leaf to compute the necessary randomness "r" and the commitment "a"
    pub fn first_message() -> (Wscalar, FirstDlogProverMessage) {
        let r = dlog_group::random_scalar_in_group_range(crypto_utils::secure_rng());
        let g = generator();
        let a = exponentiate(&g, &r);
        (r.into(), FirstDlogProverMessage { a: a.into() })
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
        let e: Scalar = challenge.clone().into();
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
        let g = generator();
        let h = *proposition.h.clone();
        let e: Scalar = challenge.clone().into();
        let g_z = exponentiate(&g, second_message.z.as_scalar_ref());
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

    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        #[cfg(feature = "arbitrary")]
        fn test_compute_commitment(secret in any::<DlogProverInput>(), challenge in any::<Challenge>()) {
            let pk = secret.public_image();
            let (r, commitment) = interactive_prover::first_message();
            let second_message = interactive_prover::second_message(&secret, r, &challenge);
            let a = interactive_prover::compute_commitment(&pk, &challenge, &second_message);
            prop_assert_eq!(a, *commitment.a);
        }
    }
}
