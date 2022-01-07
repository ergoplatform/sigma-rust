#![allow(dead_code)] // TODO: remove

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::error::SharedPeerError;
use crate::message::Request;
use crate::message::Response;
use crate::peer_connection_handler::ConnectionId;
use crate::PeerInfo;

/// The "client" duplex half of a peer connection.
pub struct Client {
    pub(crate) server_tx: mpsc::Sender<ClientRequest>,
    pub(crate) peer_info: PeerInfo,
    pub(crate) connection_id: ConnectionId,
}

#[derive(Debug)]
pub(crate) struct ClientRequest {
    /// network request for the peer.
    pub request: Request,

    /// The response channel
    pub tx: oneshot::Sender<Result<Response, SharedPeerError>>,

    /// tracing context for the request
    pub span: tracing::Span,
}
