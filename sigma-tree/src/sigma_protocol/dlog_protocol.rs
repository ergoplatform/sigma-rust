use crate::{big_integer::BigInteger, ecpoint::EcPoint};

pub struct FirstDlogProverMessage(EcPoint);
pub struct SecondDlogProverMessage(BigInteger);

pub mod interactive_prover {
    use super::{FirstDlogProverMessage, SecondDlogProverMessage};
    use crate::{
        big_integer::BigInteger,
        sigma_protocol::{Challenge, DlogProverInput, ProveDlog},
    };

    pub fn simulate(
        public_input: &ProveDlog,
        challenge: &Challenge,
    ) -> (FirstDlogProverMessage, SecondDlogProverMessage) {
        todo!()
    }

    pub fn first_message(proposition: &ProveDlog) -> (BigInteger, FirstDlogProverMessage) {
        todo!()
    }

    pub fn second_message(
        private_input: &DlogProverInput,
        rnd: BigInteger,
        challenge: &Challenge,
    ) -> SecondDlogProverMessage {
        todo!()
    }
}
