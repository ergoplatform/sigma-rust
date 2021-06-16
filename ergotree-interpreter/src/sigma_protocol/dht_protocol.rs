//! Discrete logarithm signature protocol

use super::ProverMessage;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use k256::Scalar;

/// a = g^r, b = h^r
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FirstDhTupleProverMessage {
    a: Box<EcPoint>,
    b: Box<EcPoint>,
}

impl FirstDhTupleProverMessage {
    pub fn new(a: EcPoint, b: EcPoint) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
        }
    }
}

impl ProverMessage for FirstDhTupleProverMessage {
    fn bytes(&self) -> Vec<u8> {
        let mut res = self.a.sigma_serialize_bytes();
        res.append(self.b.sigma_serialize_bytes().as_mut());
        res
    }
}

//z = r + ew mod q
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SecondDhTupleProverMessage {
    z: Scalar,
}

/// Interactive prover
pub(crate) mod interactive_prover {

    use super::*;
    use crate::sigma_protocol::private_input::DhTupleProverInput;
    use crate::sigma_protocol::Challenge;
    use ergotree_ir::sigma_protocol::dlog_group;
    use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
    use k256::Scalar;

    pub(crate) fn simulate(
        public_input: &ProveDhTuple,
        challenge: &Challenge,
    ) -> (FirstDhTupleProverMessage, SecondDhTupleProverMessage) {
        todo!()
    }

    pub(crate) fn first_message(
        public_input: &ProveDhTuple,
    ) -> (Scalar, FirstDhTupleProverMessage) {
        let r = dlog_group::random_scalar_in_group_range();
        let a = dlog_group::exponentiate(&public_input.g, &r);
        let b = dlog_group::exponentiate(&public_input.h, &r);
        (r, FirstDhTupleProverMessage::new(a, b))
    }

    pub(crate) fn second_message(
        private_input: &DhTupleProverInput,
        rnd: &Scalar,
        challenge: &Challenge,
    ) -> SecondDhTupleProverMessage {
        let e: Scalar = challenge.clone().into();
        // modulo multiplication, no need to explicit mod op
        let ew = e.mul(&private_input.w);
        // modulo addition, no need to explicit mod op
        let z = rnd.add(&ew);
        SecondDhTupleProverMessage { z }
    }
}
