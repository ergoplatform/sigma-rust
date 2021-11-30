//! Peer info types

use crate::{peer_addr::PeerAddr, peer_spec::PeerSpec, protocol_version::ProtocolVersion};

/// Direction of the connection to a peer
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ConnectionDirection {
    /// A peer is connecting to us
    Incoming,
    /// We are connecting to a peer
    Outgoing,
}

/// Information about peer to be stored in PeerDatabase
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct PeerInfo {
    /// general information about the peer
    pub peer_spec: PeerSpec,
    /// timestamp when last handshake was done
    last_handshake: u64,
    /// type of connection (Incoming/Outgoing) established to this peer if any
    conn_type: Option<ConnectionDirection>,
}

impl PeerInfo {
    /// Create new PeerInfo instance
    pub fn new(
        peer_spec: PeerSpec,
        last_handshake: u64,
        conn_type: Option<ConnectionDirection>,
    ) -> Self {
        Self {
            peer_spec,
            last_handshake,
            conn_type,
        }
    }

    /// Create peer info from address only, when we don't know other fields
    /// (e.g. we got this information from config or from API)
    pub fn from_addr(addr: PeerAddr) -> PeerInfo {
        let peer_spec = PeerSpec::new(
            "unknown",
            ProtocolVersion::INITIAL,
            &format!("unknown-{}", addr.0.to_string()),
            Some(addr),
            vec![],
        );

        PeerInfo::new(peer_spec, 0, None)
    }
}
