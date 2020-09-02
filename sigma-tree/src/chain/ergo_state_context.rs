//! Blockchain state

/// Blockchain state (last headers, etc.)
pub struct ErgoStateContext();

impl ErgoStateContext {
    /// Empty(dummy) value
    pub fn dummy() -> ErgoStateContext {
        ErgoStateContext()
    }
}
