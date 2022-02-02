//! Ergo node REST API endpoints

use ergo_chain_types::BlockId;
use ergo_nipopow::NipopowProof;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::RequestBuilder;

use crate::NodeConf;
use crate::NodeError;
use crate::NodeInfo;
use crate::PeerInfo;

fn set_req_headers(rb: RequestBuilder, node: NodeConf) -> RequestBuilder {
    rb.header("accept", "application/json")
        .header("api_key", node.get_node_api_header())
        .header(CONTENT_TYPE, "application/json")
}

fn build_client(_node_conf: &NodeConf) -> Result<Client, reqwest::Error> {
    #[cfg(not(target_arch = "wasm32"))]
    let builder = if let Some(timeout) = _node_conf.timeout {
        reqwest::Client::builder().timeout(timeout)
    } else {
        reqwest::Client::builder()
    };
    // there is no `timeout` method yet in Wasm reqwest implementation
    // see https://github.com/seanmonstar/reqwest/issues/1135
    #[cfg(target_arch = "wasm32")]
    let builder = reqwest::Client::builder();

    builder.build()
}

/// GET on /info endpoint
pub async fn get_info(node: NodeConf) -> Result<NodeInfo, NodeError> {
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join("info").unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    Ok(set_req_headers(rb, node)
        .send()
        .await?
        .json::<NodeInfo>()
        .await?)
}

/// GET on /peers/all endpoint
pub async fn get_peers_all(_node: NodeConf) -> Result<Vec<PeerInfo>, NodeError> {
    todo!()
}

/// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
pub async fn get_nipopow_proof_by_header_id(
    _node: NodeConf,
    _min_chain_length: u32,
    _suffix_len: u32,
    _header_id: BlockId,
) -> Result<NipopowProof, NodeError> {
    todo!()
}

// pub async fn get_blocks_header_id_proof_for_tx_id(
//     _node: NodeConf,
//     _header_id: BlockId,
//     _tx_id: TxId,
// ) -> Result<Option<MerkleProof>, NodeError> {
//     todo!()
// }

#[allow(clippy::unwrap_used)]
#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::time::Duration;

    use ergo_chain_types::PeerAddr;

    use super::*;

    #[test]
    fn test_get_info() {
        // let runtime_inner = tokio::runtime::Runtime::new().unwrap();
        let runtime_inner = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let node_conf = NodeConf {
            addr: PeerAddr::from_str("213.239.193.208:9053").unwrap(),
            api_key: None,
            timeout: Some(Duration::from_secs(5)),
        };
        let res = runtime_inner.block_on(async { get_info(node_conf).await.unwrap() });
        assert_eq!(res.name, "ergo-mainnet-4.0.16.1");
    }
}
