use super::{dlog_group::EcPoint, ProverMessage};
use crate::{big_integer::BigInteger, serialization::SigmaSerializable};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FirstDlogProverMessage(pub EcPoint);

impl ProverMessage for FirstDlogProverMessage {
    fn bytes(&self) -> Vec<u8> {
        self.0.sigma_serialise_bytes()
    }
}

pub struct SecondDlogProverMessage(BigInteger);

pub mod interactive_prover {
    use super::{FirstDlogProverMessage, SecondDlogProverMessage};
    use crate::{
        big_integer::BigInteger,
        sigma_protocol::{dlog_group, Challenge, DlogProverInput, ProveDlog},
    };

    pub fn simulate(
        public_input: &ProveDlog,
        challenge: &Challenge,
    ) -> (FirstDlogProverMessage, SecondDlogProverMessage) {
        todo!()
    }

    pub fn first_message(proposition: &ProveDlog) -> (BigInteger, FirstDlogProverMessage) {
        let scalar = dlog_group::random_scalar_in_group_range();
        let g = dlog_group::generator();
        let a = dlog_group::exponentiate(&g, &scalar);
        (scalar.into(), FirstDlogProverMessage(a))
    }

    pub fn second_message(
        private_input: &DlogProverInput,
        rnd: BigInteger,
        challenge: &Challenge,
    ) -> SecondDlogProverMessage {
        todo!()
    }
}
