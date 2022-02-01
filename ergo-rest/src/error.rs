use thiserror::Error;

/// Possible errors during the communication with node
#[derive(Error, Debug)]
pub enum NodeError {
    /// reqwest error
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
