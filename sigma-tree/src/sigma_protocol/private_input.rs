use super::{dlog_group, ProveDlog};
use k256::Scalar;

/// Secret key of discrete logarithm signature protocol
#[derive(PartialEq, Debug, Clone)]
pub struct DlogProverInput {
    /// secret key value
    pub w: Scalar,
}

impl DlogProverInput {
    /// generates random secret in the range [0, n), where n is DLog group order.
    pub fn random() -> DlogProverInput {
        DlogProverInput {
            w: dlog_group::random_scalar_in_group_range(),
        }
    }

    /// public key of discrete logarithm signature protocol
    pub fn public_image(&self) -> ProveDlog {
        // test it, see https://github.com/ergoplatform/sigma-rust/issues/38
        let g = dlog_group::generator();
        ProveDlog::new(dlog_group::exponentiate(&g, &self.w))
    }
}

/// Private inputs (secrets)
pub enum PrivateInput {
    /// Discrete logarithm prover input
    DlogProverInput(DlogProverInput),
    /// DH tuple prover input
    DiffieHellmanTupleProverInput,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for DlogProverInput {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![Just(DlogProverInput::random()),].boxed()
        }
    }
}
