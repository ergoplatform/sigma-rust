use crate::reqwest;
use derive_more::From;
use thiserror::Error;

/// Possible errors during the communication with node
#[derive(Error, Debug)]
pub enum NodeError {
    /// reqwest error
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    /// Invalid numerical URL segment
    #[error("Invalid numerical URL segment")]
    InvalidNumericalUrlSegment,
}

#[derive(Debug, Error, From)]
/// Peer discovery error
pub enum PeerDiscoveryError {
    /// `Url` error
    #[error("URL error")]
    UrlError,
    /// mpsc sender error
    #[error("MPSC sender error")]
    MpscSender,
    /// tokio::spawn `JoinError`
    #[error("Join error")]
    JoinError,
    /// task spawn error
    #[error("Task spawn error")]
    TaskSpawn,
    /// IO error
    #[error("IO error")]
    IO(std::io::Error),
    /// Timeout duration is too short
    #[error("Timeout duration is too short")]
    TimeoutTooShort,
    /// There aren't any node requests to be made
    #[error("There aren't any node requests to be made")]
    NoPendingNodeRequests,
}
