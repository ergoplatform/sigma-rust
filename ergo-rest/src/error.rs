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

#[derive(Debug, Error)]
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
}
