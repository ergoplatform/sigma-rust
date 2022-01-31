use ergo_chain_types::BlockId;
use ergo_chain_types::PeerAddr;
use ergo_nipopow::NipopowProof;

use crate::NodeError;
use crate::NodeInfo;
use crate::PeerInfo;

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

    /// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
    pub async fn get_nipopow_proof_by_header_id(
        _min_chain_length: u32,
        _suffix_len: u32,
        _header_id: BlockId,
    ) -> Result<NipopowProof, NodeError> {
        todo!()
    }

    // pub async fn get_blocks_header_id_proof_for_tx_id(
    //     _header_id: BlockId,
    //     _tx_id: TxId,
    // ) -> Result<Option<MerkleProof>, NodeError> {
    //     todo!()
    // }
}
