//! Ergo node REST API endpoints

use bounded_integer::BoundedU16;
use bounded_vec::NonEmptyVec;
use ergo_chain_types::BlockId;
use ergo_chain_types::Header;
use ergo_merkle_tree::MerkleProof;
use ergo_nipopow::NipopowProof;
use ergotree_ir::chain::tx_id::TxId;
use std::time::Duration;
use url::Url;

use crate::error::PeerDiscoveryError;
use crate::NodeConf;
use crate::NodeError;
use crate::NodeInfo;

use super::build_client;
use super::set_req_headers;

#[cfg(target_arch = "wasm32")]
pub use crate::api::peer_discovery_internals::ChromePeerDiscoveryScan;

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

/// GET on /blocks/{header_id}/header endpoint
pub async fn get_header(node: NodeConf, header_id: BlockId) -> Result<Header, NodeError> {
    let header_str = String::from(header_id.0);
    let mut path = "blocks/".to_owned();
    path.push_str(&header_str);
    path.push_str("/header");
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join(&path).unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    Ok(set_req_headers(rb, node)
        .send()
        .await?
        .json::<Header>()
        .await?)
}

/// Given a list of seed nodes, search for peer nodes with an active REST API on port 9053.
///  - `seeds` represents a list of ergo node URLs from which to start peer discovery.
///  - `max_parallel_tasks` represents the maximum number of tasks to spawn for ergo node HTTP
///    requests. Note that the actual number of parallel HTTP requests may well be higher than this
///    number.
///  - `timeout` represents the amount of time that is spent search for peers. Once the timeout
///    value is reached, return with the vec of active peers that have been discovered up to that
///    point in time.
///
/// IMPORTANT: do not call this function on Chromium, as it will likely mess with the browser's
/// ability to make HTTP requests. Use `peer_discovery_chrome` instead. For more information why
/// please refer to the module documentation for `crate::api::peer_discovery_internals::chrome`.
pub async fn peer_discovery(
    seeds: NonEmptyVec<Url>,
    max_parallel_tasks: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    super::peer_discovery_internals::peer_discovery_inner(seeds, max_parallel_tasks, timeout).await
}

#[cfg(target_arch = "wasm32")]
/// Given a list of seed nodes, search for peer nodes with an active REST API on port 9053.
///  - `seeds` represents a list of ergo node URLs from which to start peer discovery.
///  - `max_parallel_requests` represents the maximum number of HTTP requests that can be made in
///    parallel
///  - `timeout` represents the amount of time that is spent searching for peers PLUS a waiting
///    period of 80 seconds to give Chrome the time to relinquish failed preflight requests. Must be
///    at least 90 seconds. Once the timeout value is reached, return with the vec of active peers
///    that have been discovered up to that point in time.
///
/// NOTE: intended to be used only on Chromium based browsers. It works on Firefox and Safari, but
/// using `peer_discovery` above gives better performance.
pub async fn peer_discovery_chrome(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    let scan = super::peer_discovery_internals::ChromePeerDiscoveryScan::new(seeds);
    super::peer_discovery_internals::peer_discovery_inner_chrome(
        scan,
        max_parallel_requests,
        timeout,
    )
    .await
    .map(|scan| scan.active_peers())
}

#[cfg(target_arch = "wasm32")]
/// An incremental (reusable) version of [`peer_discovery_chrome`] which allows for peer discovery
/// to be split into separate sub-tasks.
///
/// NOTE: intended to be used only on Chromium based browsers. It works on Firefox and Safari, but
/// using `peer_discovery` above gives better performance.
pub async fn incremental_peer_discovery_chrome(
    scan: ChromePeerDiscoveryScan,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<ChromePeerDiscoveryScan, PeerDiscoveryError> {
    super::peer_discovery_internals::peer_discovery_inner_chrome(
        scan,
        max_parallel_requests,
        timeout,
    )
    .await
}

/// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
pub async fn get_nipopow_proof_by_header_id(
    node: NodeConf,
    min_chain_length: u32,
    suffix_len: u32,
    header_id: BlockId,
) -> Result<NipopowProof, NodeError> {
    if min_chain_length == 0 || suffix_len == 0 {
        return Err(NodeError::InvalidNumericalUrlSegment);
    }
    let header_str = String::from(header_id.0);
    let mut path = "nipopow/proof/".to_owned();
    path.push_str(&min_chain_length.to_string());
    path.push('/');
    path.push_str(&suffix_len.to_string());
    path.push('/');
    path.push_str(&header_str);
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join(&path).unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    Ok(set_req_headers(rb, node)
        .send()
        .await?
        .json::<NipopowProof>()
        .await?)
}

/// GET on /blocks/{header_id}/proofFor/{tx_id} to request the merkle proof for a given transaction
/// that belongs to the given header ID.
pub async fn get_blocks_header_id_proof_for_tx_id(
    node: NodeConf,
    header_id: BlockId,
    tx_id: TxId,
) -> Result<Option<MerkleProof>, NodeError> {
    let header_str = String::from(header_id.0);
    let mut path = "blocks/".to_owned();
    path.push_str(&header_str);
    path.push_str("/proofFor/");
    let tx_id_str = String::from(tx_id);
    path.push_str(&tx_id_str);
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join(&path).unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    Ok(set_req_headers(rb, node)
        .send()
        .await?
        .json::<Option<MerkleProof>>()
        .await?)
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::str::FromStr;
    use std::time::Duration;

    use ergo_chain_types::PeerAddr;

    use super::*;

    #[test]
    fn test_get_info() {
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
        assert_ne!(res.name, "");
    }

    #[test]
    fn test_get_nipopow_proof_by_header_id() {
        use ergo_chain_types::{BlockId, Digest32};
        let header_id = BlockId(
            Digest32::try_from(String::from(
                "9bcb535c2d05fbced6de3d73c63337d6deb64af387438fa748d66ddf3d33ee89",
            ))
            .unwrap(),
        );
        let runtime_inner = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let node_conf = NodeConf {
            addr: PeerAddr::from_str("213.239.193.208:9053").unwrap(),
            api_key: None,
            timeout: Some(Duration::from_secs(5)),
        };
        let m = 7;
        let k = 6;
        let res = runtime_inner.block_on(async {
            get_nipopow_proof_by_header_id(node_conf, m, k, header_id)
                .await
                .unwrap()
        });
        assert_eq!(res.suffix_head.header.id, header_id);
        assert!(!res.prefix.is_empty());
        assert_eq!(res.m, m);
        assert_eq!(res.k, k);
    }

    #[test]
    fn test_peer_discovery() {
        let seeds: Vec<_> = [
            "http://213.239.193.208:9030",
            "http://159.65.11.55:9030",
            "http://165.227.26.175:9030",
            "http://159.89.116.15:9030",
            "http://136.244.110.145:9030",
            "http://94.130.108.35:9030",
            "http://51.75.147.1:9020",
            "http://221.165.214.185:9030",
            "http://51.81.185.231:9031",
            "http://217.182.197.196:9030",
            "http://62.171.190.193:9030",
            "http://173.212.220.9:9030",
            "http://176.9.65.58:9130",
            "http://213.152.106.56:9030",
        ]
        .iter()
        .map(|s| Url::from_str(s).unwrap())
        .collect();
        let runtime_inner = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let (res_with_quick_timeout, res_with_longer_timeout) = runtime_inner.block_on(async {
            let res_quick = peer_discovery(
                NonEmptyVec::from_vec(seeds.clone()).unwrap(),
                BoundedU16::new(5).unwrap(),
                Duration::from_millis(1000),
            )
            .await
            .unwrap();

            tokio::time::sleep(Duration::from_secs(5)).await;

            let res_long = peer_discovery(
                NonEmptyVec::from_vec(seeds).unwrap(),
                BoundedU16::new(5).unwrap(),
                Duration::from_millis(10000),
            )
            .await
            .unwrap();
            (res_quick, res_long)
        });
        println!(
            "{} quick peers, {} long peers",
            res_with_quick_timeout.len(),
            res_with_longer_timeout.len()
        );
        println!("discovered: {:?}", res_with_longer_timeout);
        assert!(!res_with_longer_timeout.is_empty());
        assert!(res_with_quick_timeout.len() <= res_with_longer_timeout.len());
    }
}
