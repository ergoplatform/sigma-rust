use ergo_chain_types::PeerAddr;

use crate::NodeError;
use crate::NodeInfo;

/// Known nodes that are serving REST API
pub struct KnownNodes {
    nodes: Vec<NodeInfo>,
    // to ignore during peer discovery
    p2p_only_nodes: Vec<PeerAddr>,
}

impl KnownNodes {
    /// Load node addresses that serve REST API
    pub fn load_from_addr(_addrs: Vec<PeerAddr>) {
        todo!()
    }

    /// Get the known nodes
    pub fn get_nodes(&self) -> Vec<NodeInfo> {
        todo!()
    }

    /// Load from the full nodes info (previously discovered and persisted from [`KnownNodes::get_nodes`])
    pub fn load_nodes(_nodes: Vec<NodeInfo>) {
        todo!()
    }

    /// Ask known nodes for new peers until `target_new_discovered` new nodes are discovered
    pub async fn discover(
        &self,
        _max_parallel_req: usize,
        _target_new_discovered: usize,
    ) -> Result<(), NodeError> {
        todo!()
    }
}
