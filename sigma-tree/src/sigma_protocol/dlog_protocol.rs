use super::{dlog_group::EcPoint, ProverMessage};
use crate::serialization::SigmaSerializable;
use k256::Scalar;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FirstDlogProverMessage(pub EcPoint);

impl From<EcPoint> for FirstDlogProverMessage {
    fn from(ecp: EcPoint) -> Self {
        FirstDlogProverMessage(ecp)
    }
}

impl ProverMessage for FirstDlogProverMessage {
    fn bytes(&self) -> Vec<u8> {
        self.0.sigma_serialise_bytes()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct SecondDlogProverMessage(pub Scalar);

pub mod interactive_prover {
    use super::{FirstDlogProverMessage, SecondDlogProverMessage};
    use crate::sigma_protocol::{dlog_group, Challenge, DlogProverInput, ProveDlog};
    use dlog_group::EcPoint;
    use k256::Scalar;

    pub fn simulate(
        public_input: &ProveDlog,
        challenge: &Challenge,
    ) -> (FirstDlogProverMessage, SecondDlogProverMessage) {
        todo!()
    }

    pub fn first_message(proposition: &ProveDlog) -> (Scalar, FirstDlogProverMessage) {
        let scalar = dlog_group::random_scalar_in_group_range();
        let g = dlog_group::generator();
        let a = dlog_group::exponentiate(&g, &scalar);
        (scalar, FirstDlogProverMessage(a))
    }

    pub fn second_message(
        private_input: &DlogProverInput,
        rnd: Scalar,
        challenge: &Challenge,
    ) -> SecondDlogProverMessage {
        let e: Scalar = challenge.clone().into();
        // modulo multiplication, no need to explicit mod op
        let ew = e.mul(&private_input.w);
        // modulo addition, no need to explicit mod op
        let z = rnd.add(&ew);
        SecondDlogProverMessage(z)
    }

    /**
     * The function computes initial prover's commitment to randomness
     * ("a" message of the sigma-protocol) based on the verifier's challenge ("e")
     * and prover's response ("z")
     *
     * g^z = a*h^e => a = g^z/h^e
     */
    pub fn compute_commitment(
        proposition: &ProveDlog,
        challenge: &Challenge,
        second_message: &SecondDlogProverMessage,
    ) -> EcPoint {
        todo!()
    }
}
