//! Discrete logarithm signature protocol

use super::ProverMessage;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use k256::Scalar;

/// First message from the prover (message `a` of `SigmaProtocol`) for discrete logarithm case
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FirstDlogProverMessage(pub(crate) EcPoint);

impl From<EcPoint> for FirstDlogProverMessage {
    fn from(ecp: EcPoint) -> Self {
        FirstDlogProverMessage(ecp)
    }
}

impl ProverMessage for FirstDlogProverMessage {
    fn bytes(&self) -> Vec<u8> {
        self.0.sigma_serialize_bytes()
    }
}

/// Second message from the prover (message `z` of `SigmaProtocol`) for discrete logarithm case
#[derive(PartialEq, Debug, Clone)]
pub struct SecondDlogProverMessage {
    /// message `z`
    pub z: Scalar,
}

impl From<Scalar> for SecondDlogProverMessage {
    fn from(z: Scalar) -> Self {
        SecondDlogProverMessage { z }
    }
}

/// Interactive prover
pub(crate) mod interactive_prover {
    use std::ops::Mul;

    use super::{FirstDlogProverMessage, SecondDlogProverMessage};
    use crate::sigma_protocol::{private_input::DlogProverInput, Challenge};
    use ergotree_ir::sigma_protocol::dlog_group;
    use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
    use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
    use k256::Scalar;

    pub(crate) fn simulate(
        public_input: &ProveDlog,
        challenge: &Challenge,
    ) -> (FirstDlogProverMessage, SecondDlogProverMessage) {
        //SAMPLE a random z <- Zq
        let z = dlog_group::random_scalar_in_group_range();

        //COMPUTE a = g^z*h^(-e)  (where -e here means -e mod q)
        let e: Scalar = challenge.clone().into();
        let minus_e = e.negate();
        let h_to_e = dlog_group::exponentiate(&public_input.h, &minus_e);
        let g_to_z = dlog_group::exponentiate(&dlog_group::generator(), &z);
        let a = g_to_z * &h_to_e;
        (FirstDlogProverMessage(a), SecondDlogProverMessage { z })
    }

    /// Create first message from the prover and a randomness
    pub(crate) fn first_message() -> (Scalar, FirstDlogProverMessage) {
        let r = dlog_group::random_scalar_in_group_range();
        let g = dlog_group::generator();
        let a = dlog_group::exponentiate(&g, &r);
        (r, FirstDlogProverMessage(a))
    }

    /// Create second message from the prover
    pub(crate) fn second_message(
        private_input: &DlogProverInput,
        rnd: Scalar,
        challenge: &Challenge,
    ) -> SecondDlogProverMessage {
        let e: Scalar = challenge.clone().into();
        // modulo multiplication, no need to explicit mod op
        let ew = e.mul(&private_input.w);
        // modulo addition, no need to explicit mod op
        let z = rnd.add(&ew);
        z.into()
    }

    /**
     * The function computes initial prover's commitment to randomness
     * ("a" message of the sigma-protocol) based on the verifier's challenge ("e")
     * and prover's response ("z")
     *
     * g^z = a*h^e => a = g^z/h^e
     */
    pub(crate) fn compute_commitment(
        proposition: &ProveDlog,
        challenge: &Challenge,
        second_message: &SecondDlogProverMessage,
    ) -> EcPoint {
        let g = dlog_group::generator();
        let h = *proposition.h.clone();
        let e: Scalar = challenge.clone().into();
        let g_z = dlog_group::exponentiate(&g, &second_message.z);
        let h_e = dlog_group::exponentiate(&h, &e);
        g_z * &dlog_group::inverse(&h_e)
    }
}

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
            prop_assert_eq!(a, commitment.0);
        }
    }
}
