use ergo_chain_types::PeerAddr;

/// Node extended information from /info REST API endpoint
pub struct NodeInfo {
    /// Node's REST API address
    http_addr: PeerAddr,
}
