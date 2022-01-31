use ergo_chain_types::PeerAddr;

/// Ergo node configuration
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct NodeConf {
    /// Node address
    pub addr: PeerAddr,
    /// Node API key
    pub api_key: Option<&'static str>,
}
