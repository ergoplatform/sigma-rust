use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures::Future;
use tokio::net::TcpStream;
use tower::Service;

use crate::error::BoxError;
use crate::peer_info::ConnectionDirection;
use crate::Client;
use crate::PeerAddr;

/// A service that handshakes with a remote peer and constructs a client/server pair.
#[derive(Clone)]
pub struct PeerConnectionHandler {}

impl Service<HandshakeRequest> for PeerConnectionHandler {
    type Response = Client;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, req: HandshakeRequest) -> Self::Future {
        todo!()
    }
}

pub struct HandshakeRequest {
    pub tcp_stream: TcpStream,
    pub connected_addr: ConnectionId,
}

/// Wraps (remoteAddress, localAddress, direction) tuple, which allows to precisely identify peer.
#[allow(dead_code)] // TODO: remove
pub struct ConnectionId {
    remote_address: PeerAddr,
    // local_address: PeerAddr,
    direction: ConnectionDirection,
}
impl ConnectionId {
    pub(crate) fn new_outbound_direct(addr: PeerAddr) -> Self {
        todo!()
    }
}
