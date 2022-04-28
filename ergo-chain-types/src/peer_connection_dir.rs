use serde::{Deserialize, Serialize};

/// Direction of the connection to a peer
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash, Deserialize, Serialize)]
pub enum ConnectionDirection {
    /// A peer is connecting to us
    Incoming,
    /// We are connecting to a peer
    Outgoing,
}
