use futures::channel::oneshot;
use std::sync::mpsc;

use crate::error::SharedPeerError;
use crate::message::Request;
use crate::message::Response;

/// The "client" duplex half of a peer connection.
pub struct Client {
    #[allow(dead_code)] // TODO: remove
    pub(crate) server_tx: mpsc::Sender<ClientRequest>,
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
