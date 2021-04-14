/// Block header with the current `spendingTransaction`, that can be predicted
/// by a miner before it's formation
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PreHeader {
    /// Block height
    pub height: i32,
}

impl PreHeader {
    /// Dummy instance intended for tests where actual values are not used
    pub fn dummy() -> Self {
        PreHeader { height: 0 }
    }
}