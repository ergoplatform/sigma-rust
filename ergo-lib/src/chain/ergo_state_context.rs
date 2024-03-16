//! Blockchain state
use ergo_chain_types::{Header, PreHeader};

use super::parameters::Parameters;

/// Fixed number of last block headers in descending order (first header is the newest one)
pub type Headers = [Header; 10];

/// Blockchain state (last headers, etc.)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoStateContext {
    /// Block header with the current `spendingTransaction`, that can be predicted
    /// by a miner before it's formation
    pub pre_header: PreHeader,
    /// Fixed number of last block headers in descending order (first header is the newest one)
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
mod arbitrary {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for ErgoStateContext {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            // TODO: parameters should implement arbitrary as well, based on minimum/maximum constraints of each parameter
            (any::<PreHeader>(), any::<Headers>())
                .prop_map(|(pre_header, headers)| {
                    Self::new(pre_header, headers, Parameters::default())
                })
                .boxed()
        }
    }
}
