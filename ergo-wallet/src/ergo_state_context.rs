/// TBD
pub struct ErgoStateContext();

impl ErgoStateContext {
    /// Dummy "empty" context
    pub fn dummy() -> ErgoStateContext {
        ErgoStateContext()
    }

    /// build from last block headers from node REST API "blocks/lastHeaders/{count}"
    pub fn from_last_headers(_json_str: &str) -> Result<ErgoStateContext, ErgoStateContextError> {
        todo!()
    }
}

/// Errors on parsing and building from node REST API blocks/lastHeaders
pub enum ErgoStateContextError {}
