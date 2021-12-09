use crate::peer_connection_handler::PeerConnectionHandler;

/// Opens a TCP connection before forwarding to the inner peer connection handling service for a handshake.
#[allow(dead_code)]
pub struct PeerConnector {
    handshaker: PeerConnectionHandler,
}

// TODO: impl Service trait
