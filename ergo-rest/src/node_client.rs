use ergo_chain_types::PeerAddr;

use crate::NodeInfo;
use crate::PeerInfo;

/// Possible errors during the communication with node
pub enum NodeError {}

/// Ergo node info and methods for sending requests
pub struct NodeClient {
    /// Node address
    pub addr: PeerAddr,
    /// Node API key
    pub api_key: Option<String>,
}

impl NodeClient {
    /// GET on /info endpoint
    pub async fn get_info(&self) -> Result<NodeInfo, NodeError> {
        todo!()
    }

    /// GET on /peers/all endpoint
    pub async fn get_peers_all(&self) -> Result<Vec<PeerInfo>, NodeError> {
        todo!()
    }
}
