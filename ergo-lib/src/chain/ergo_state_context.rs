//! Blockchain state
use bounded_vec::BoundedVec;
use ergo_chain_types::{Header, PreHeader};

use super::parameters::Parameters;

/// Last block headers in descending order (first header is the newest one).
/// Between 1 and 10: the SDK signs and validates against an existing chain tip,
/// so at least the newest header is always available (the script context itself
/// allows fewer — see `ergotree_ir::chain::context::ContextHeaders`). A node
/// near genesis supplies as many real headers as exist instead of padding.
pub type Headers = BoundedVec<Header, 1, 10>;

/// Blockchain state (last headers, etc.)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoStateContext {
    /// Block header with the current `spendingTransaction`, that can be predicted
    /// by a miner before it's formation
    pub pre_header: PreHeader,
    /// Last block headers in descending order (first header is the newest one)
    pub headers: Headers,
    /// Parameters that can be adjusted by voting
    pub parameters: Parameters,
}

impl ErgoStateContext {
    /// Create an ErgoStateContext instance
    /// # Parameters
    /// For signing, [Parameters::default()] is sufficient. For consensus-critical applications that validate transactions it is important that parameters represent the latest state of the blockchain
    pub fn new(
        pre_header: PreHeader,
        headers: Headers,
        parameters: Parameters,
    ) -> ErgoStateContext {
        ErgoStateContext {
            pre_header,
            headers,
            parameters,
        }
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use proptest::{collection::vec, prelude::*};

    impl Arbitrary for ErgoStateContext {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            // TODO: parameters should implement arbitrary as well, based on minimum/maximum constraints of each parameter
            (any::<PreHeader>(), vec(any::<Header>(), 10))
                .prop_map(|(pre_header, headers)| {
                    Self::new(
                        pre_header,
                        headers.try_into().unwrap(),
                        Parameters::default(),
                    )
                })
                .boxed()
        }
    }
}
