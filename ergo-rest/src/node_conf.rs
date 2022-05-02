use std::time::Duration;

use crate::reqwest::header::HeaderValue;
use ergo_chain_types::PeerAddr;

/// Ergo node configuration
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct NodeConf {
    /// Node address
    pub addr: PeerAddr,
    /// Node API key
    pub api_key: Option<&'static str>,
    /// Request timeout
    pub timeout: Option<Duration>,
}

impl NodeConf {
    /// Generate the value for api_key header key
    pub fn get_node_api_header(&self) -> HeaderValue {
        match self.api_key {
            Some(api_key) => match HeaderValue::from_str(api_key) {
                Ok(k) => k,
                _ => HeaderValue::from_static("None"),
            },
            None => HeaderValue::from_static("None"),
        }
    }
}
