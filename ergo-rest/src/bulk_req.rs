use crate::reqwest::{Request, Response};
use ergo_chain_types::BlockId;
use ergo_chain_types::PeerAddr;
use ergo_nipopow::NipopowProof;

use crate::NodeError;

async fn bulk_req(_nodes: Vec<PeerAddr>, _req: Request) -> Result<Vec<Response>, NodeError> {
    todo!()
}

pub struct HostParams {
    pub nodes: Vec<PeerAddr>,
    pub max_parallel_req: usize,
}

pub async fn bulk_get_nipopow_proof_by_header_id(
    _host_params: &HostParams,
    _min_chain_length: u32,
    _suffix_len: u32,
    _header_id: BlockId,
) -> Result<Vec<NipopowProof>, NodeError> {
    todo!()
}
