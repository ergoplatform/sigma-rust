use std::time::SystemTime;

use crate::PeerSpec;

/// Network message to be send when nodes establish a new connection.
/// When a node creates an outgoing connection, it will immediately advertise its Handshake.
/// The remote node will respond with its Handshake.
/// No further communication is possible until both peers have exchanged their handshakes.
/// peerSpec - general (declared) information about peer
/// time     - handshake time
#[allow(dead_code)] // TODO: remove
pub struct Handshake {
    pub peer_spec: PeerSpec,
    pub time: SystemTime,
}
