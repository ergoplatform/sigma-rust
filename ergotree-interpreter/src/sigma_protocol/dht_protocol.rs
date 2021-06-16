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
    #[allow(clippy::unwrap_used)] // since only EcPoint is serialized here it's safe to unwrap
    fn bytes(&self) -> Vec<u8> {
        let mut res = self.a.sigma_serialize_bytes().unwrap();
        res.append(self.b.sigma_serialize_bytes().unwrap().as_mut());
        res
    }
}

//z = r + ew mod q
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SecondDhTupleProverMessage {
    pub z: Scalar,
}

/// Interactive prover
pub(crate) mod interactive_prover {

    use std::ops::Mul;

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
        //SAMPLE a random z <- Zq
        let z = dlog_group::random_scalar_in_group_range();

        // COMPUTE a = g^z*u^(-e) and b = h^z*v^{-e}  (where -e here means -e mod q)
        let e: Scalar = challenge.clone().into();
        let minus_e = e.negate();
        let h_to_z = dlog_group::exponentiate(&public_input.h, &z);
        let g_to_z = dlog_group::exponentiate(&public_input.g, &z);
        let u_to_minus_e = dlog_group::exponentiate(&public_input.u, &minus_e);
        let v_to_minus_e = dlog_group::exponentiate(&public_input.v, &minus_e);
        let a = g_to_z.mul(&u_to_minus_e);
        let b = h_to_z.mul(&v_to_minus_e);
        (
            FirstDhTupleProverMessage::new(a, b),
            SecondDhTupleProverMessage { z },
        )
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

    /// The function computes initial prover's commitment to randomness
    /// ("a" message of the sigma-protocol, which in this case has two parts "a" and "b")
    /// based on the verifier's challenge ("e")
    /// and prover's response ("z")
    ///
    /// g^z = a*u^e, h^z = b*v^e  => a = g^z/u^e, b = h^z/v^e
    #[allow(clippy::many_single_char_names)]
    pub(crate) fn compute_commitment(
        proposition: &ProveDhTuple,
        challenge: &Challenge,
        second_message: &SecondDhTupleProverMessage,
    ) -> (EcPoint, EcPoint) {
        let g = proposition.g.clone();
        let h = proposition.h.clone();
        let u = proposition.u.clone();
        let v = proposition.v.clone();

        let z = second_message.z;

        let e: Scalar = challenge.clone().into();

        let g_to_z = dlog_group::exponentiate(&g, &z);
        let h_to_z = dlog_group::exponentiate(&h, &z);

        let u_to_e = dlog_group::exponentiate(&u, &e);
        let v_to_e = dlog_group::exponentiate(&v, &e);

        let a = g_to_z.mul(&dlog_group::inverse(&u_to_e));
        let b = h_to_z.mul(&dlog_group::inverse(&v_to_e));
        (a, b)
    }
}
