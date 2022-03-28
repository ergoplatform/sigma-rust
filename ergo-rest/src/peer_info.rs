use ergo_chain_types::ConnectionDirection;
use ergo_chain_types::PeerAddr;

/// Peer info returned by node REST API
pub struct PeerInfo {
    /// Peer address
    pub addr: PeerAddr,
    /// Timestamp of the last handshake
    pub last_handshake: u64, // TODO: any more precise type?
    /// Node name
    pub name: String,
    /// Type of connection (Incoming/Outgoing) established to this peer if any
    pub conn_type: Option<ConnectionDirection>,
}
