use std::collections::HashMap;

use ergo_chain_types::PeerAddr;

use crate::NodeError;
use crate::NodeInfo;

/// Known nodes that are serving REST API
pub struct KnownNodes {
    http_nodes: Vec<PeerAddr>,
    http_nodes_last_req: HashMap<PeerAddr, u64>,
    // to ignore/skip during peer discovery
    p2p_only_nodes: Vec<PeerAddr>,
}

impl KnownNodes {
    /// Add node addresses that serve REST API
    pub fn add(_addrs: Vec<PeerAddr>) {
        todo!()
    }

    /// Export known nodes as serialized bytes
    pub fn export(&self) -> Vec<u8> {
        todo!()
    }

    /// Load known nodes from serialized bytes (previously exported with [`KnownNodes::export`])
    pub fn import(_bytes: Vec<u8>) {
        todo!()
    }

    /// Return all known nodes
    pub fn get_all(&self) -> Vec<PeerAddr> {
        todo!()
    }

    /// Ask known nodes for new nodes until `target_new_discovered` new nodes are discovered
    /// or until we run out of nodes to ask
    /// Adds new nodes to the internal list of known nodes and returns them
    pub async fn discover_new_nodes(
        &self,
        _max_parallel_req: usize,
        _target_new_discovered: usize,
    ) -> Result<Vec<NodeInfo>, NodeError> {
        // TODO: run NodeClient::get_peers_all() in paralellel on known nodes
        // TODO: run NodeClient::get_info() for every new discovered node to confirm that it serves REST API
        todo!()
    }
}
