use futures::channel::oneshot;
use std::sync::mpsc;

use crate::error::SharedPeerError;
use crate::message::Request;
use crate::message::Response;

/// The "client" duplex half of a peer connection.
pub struct Client {
    #[allow(dead_code)]
    pub(crate) server_tx: mpsc::Sender<ClientRequest>,
}

#[derive(Debug)]
pub(crate) struct ClientRequest {
    /// network request for the peer.
    pub request: Request,

    /// The response channel, included because `peer::Client::call` returns a
    /// future that may be moved around before it resolves.
    pub tx: oneshot::Sender<Result<Response, SharedPeerError>>,

    /// tracing context for the request
    pub span: tracing::Span,
}
