use std::sync::Arc;
use thiserror::Error;
use tracing_error::TracedError;

/// A wrapper around `Arc<PeerError>` that implements `Error`.
#[derive(Error, Debug, Clone)]
#[error(transparent)]
pub struct SharedPeerError(Arc<TracedError<PeerError>>);

impl<E> From<E> for SharedPeerError
where
    PeerError: From<E>,
{
    fn from(source: E) -> Self {
        Self(Arc::new(TracedError::from(PeerError::from(source))))
    }
}

/// An error related to peer connection handling.
#[derive(Error, Debug)]
pub enum PeerError {}

/// Type alias to make working with tower traits easier.
///
/// Note: the 'static lifetime bound means that the *type* cannot have any
/// non-'static lifetimes, (e.g., when a type contains a borrow and is
/// parameterized by 'a), *not* that the object itself has 'static lifetime.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// An error during a handshake with a remote peer.
#[derive(Error, Debug)]
pub enum HandshakeError {
    /// Sending or receiving a message timed out.
    #[error("Timeout when sending or receiving a message to peer")]
    Timeout,
}

impl From<tokio::time::error::Elapsed> for HandshakeError {
    fn from(_source: tokio::time::error::Elapsed) -> Self {
        HandshakeError::Timeout
    }
}
