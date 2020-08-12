use super::{dlog_group::EcPoint, ProverMessage};
use crate::serialization::SigmaSerializable;
use k256::Scalar;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FirstDlogProverMessage(pub EcPoint);

impl ProverMessage for FirstDlogProverMessage {
    fn bytes(&self) -> Vec<u8> {
        self.0.sigma_serialise_bytes()
    }
}

pub struct SecondDlogProverMessage(pub Scalar);

pub mod interactive_prover {
    use super::{FirstDlogProverMessage, SecondDlogProverMessage};
    use crate::sigma_protocol::{dlog_group, Challenge, DlogProverInput, ProveDlog};
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
        let ew = e.mul(&private_input.w);
        let z = rnd.add(&ew);
        SecondDlogProverMessage(z)
    }
}
