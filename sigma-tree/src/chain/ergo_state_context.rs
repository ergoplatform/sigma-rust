//! Blockchain state

/// Blockchain state (last headers, etc.)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoStateContext();

impl ErgoStateContext {
    /// Empty(dummy) value
    pub fn dummy() -> ErgoStateContext {
        ErgoStateContext()
    }
}
