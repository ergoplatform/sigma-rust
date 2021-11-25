//! Blockchain state
use ergotree_ir::chain::header::Header;
use ergotree_ir::chain::preheader::PreHeader;

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
}

impl ErgoStateContext {
    /// Create an ErgoStateContext instance
    pub fn new(pre_header: PreHeader, headers: Headers) -> ErgoStateContext {
        ErgoStateContext {
            pre_header,
            headers,
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
            (any::<PreHeader>(), any::<Headers>())
                .prop_map(|(pre_header, headers)| Self::new(pre_header, headers))
                .boxed()
        }
    }
}
