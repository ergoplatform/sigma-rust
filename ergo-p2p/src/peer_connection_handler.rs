use chrono::Utc;
use ergo_chain_types::ConnectionDirection;
use ergo_chain_types::PeerAddr;
use futures::Future;
use futures::FutureExt;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinError;
use tokio::time::timeout;
use tokio_util::codec::Framed;
use tower::Service;
use tracing::debug;
use tracing::debug_span;
use tracing::Instrument;

use crate::codec::Codec;
use crate::constants;
use crate::error::BoxError;
use crate::error::HandshakeError;
use crate::message::Handshake;
use crate::Client;
use crate::PeerInfo;

/// A service that handshakes with a remote peer and constructs a client/server pair.
#[derive(Clone)]
pub struct PeerConnectionHandler {}

impl Service<HandshakeRequest> for PeerConnectionHandler {
    type Response = Client;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HandshakeRequest) -> Self::Future {
        let negotiator_span = debug_span!("negotiator", conn = ?req.connection_id);
        let fut = async move {
            let (server_tx, _server_rx) = mpsc::channel(0);
            debug!( conn = ?req.connection_id, "handshake with remote peer");
            let mut peer_conn = Framed::new(req.tcp_stream, Codec::default());
            let peer_handshake = timeout(
                constants::HANDSHAKE_TIMEOUT,
                send_receive_handshake(&mut peer_conn),
            )
            .await??;
            let last_handshake = Utc::now().timestamp();
            let peer_info = PeerInfo::new(
                peer_handshake.peer_spec,
                last_handshake as u64,
                Some(ConnectionDirection::Outgoing),
            );
            let client = Client {
                server_tx,
                peer_info,
                connection_id: req.connection_id,
            };
            Ok(client)
        };
        // Spawn a new task to drive this handshake.
        tokio::spawn(fut.instrument(negotiator_span))
            .map(|x: Result<Result<Client, HandshakeError>, JoinError>| Ok(x??))
            .boxed()
    }
}

async fn send_receive_handshake(
    _peer_conn: &mut Framed<TcpStream, Codec>,
) -> Result<Handshake, HandshakeError> {
    // TODO: do a handshake
    todo!()
}

pub struct HandshakeRequest {
    pub tcp_stream: TcpStream,
    pub connection_id: ConnectionId,
}

/// Wraps (remoteAddress, localAddress, direction) tuple, which allows to precisely identify peer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ConnectionId {
    remote_address: PeerAddr,
    // local_address: PeerAddr,
    direction: ConnectionDirection,
}
impl ConnectionId {
    pub(crate) fn new_outbound_direct(_addr: PeerAddr) -> Self {
        todo!()
    }
}
