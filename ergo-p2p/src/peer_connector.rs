use std::net::SocketAddr;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures::Future;
use futures::FutureExt;
use tokio::net::TcpStream;
use tower::discover::Change;
use tower::Service;
use tower::ServiceExt;

use crate::error::BoxError;
use crate::peer_connection_handler::ConnectionId;
use crate::peer_connection_handler::HandshakeRequest;
use crate::peer_connection_handler::PeerConnectionHandler;
use crate::Client;
use crate::PeerAddr;

/// Opens a TCP connection before forwarding to the inner peer connection handling service for a handshake.
#[allow(dead_code)] // TODO: remove
pub struct PeerConnector {
    handshaker: PeerConnectionHandler,
}

/// A connector request.
/// Contains the information needed to make an outbound connection to the peer.
pub struct OutboundConnectorRequest {
    /// The listener address of the peer.
    pub addr: PeerAddr,
}

impl Service<OutboundConnectorRequest> for PeerConnector {
    type Response = Change<PeerAddr, Client>;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: OutboundConnectorRequest) -> Self::Future {
        let connection_id = ConnectionId::new_outbound_direct(req.addr);
        let addr: SocketAddr = req.addr.into();
        let mut handshaker = self.handshaker.clone();
        async move {
            let stream = TcpStream::connect(addr).await?;
            handshaker.ready().await?;
            let client = handshaker
                .call(HandshakeRequest {
                    tcp_stream: stream,
                    connected_addr: connection_id,
                })
                .await?;
            Ok(Change::Insert(req.addr, client))
        }
        .boxed()
    }
}
