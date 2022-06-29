import { expect, assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
let active_peers;
before(async () => {
    ergo_wasm = await ergo;

    // Peer discovery is performed before any of the tests begin.
    console.log("Starting peer discovery");
    const seeds = get_ergo_node_seeds();

    // Limit to 150 simultaneous HTTP requests and search for peers for 140 seconds (remember
    // there's an unavoidable waiting time of 80 seconds, to give Chrome time to relinquish failed
    // preflight requests)
    let is_chrome = true;
    active_peers = await ergo_wasm.peer_discovery(seeds, 150, 120, is_chrome);
});

// Note that the REST API tests are here due to the WASM implementation of `reqwest-wrap`. In
// particular the timeout functionality for HTTP requests requires the window object from the
// web APIs, thus requiring a web browser to run.

it('node REST API: peer_discovery endpoint (INCREMENTAL VERSION)', async () => {
    const seeds = get_ergo_node_seeds();
    let scan = new ergo_wasm.ChromePeerDiscoveryScan(seeds);

    scan = await ergo_wasm.incremental_peer_discovery_chrome(scan, 150, 90);
    let scan_1_len = scan.active_peers().len();
    scan = await ergo_wasm.incremental_peer_discovery_chrome(scan, 150, 480);
    let scan_2_len = scan.active_peers().len();
    assert(scan_1_len < scan_2_len, "Should have found more peers after second scan!");
});

it('node REST API: get_nipopow_proof_by_header_id endpoint', async () => {
    let node_conf = new ergo_wasm.NodeConf(active_peers.get(0));
    assert(node_conf != null);
    const header_id = ergo_wasm.BlockId.from_str("4caa17e62fe66ba7bd69597afdc996ae35b1ff12e0ba90c22ff288a4de10e91b");
    let res = await ergo_wasm.get_nipopow_proof_by_header_id(node_conf, 3, 4, header_id);
    assert(res != null);
    assert(node_conf != null);
});

it('node REST API: example SPV workflow', async () => {
    const header_id = ergo_wasm.BlockId.from_str("d1366f762e46b7885496aaab0c42ec2950b0422d48aec3b91f45d4d0cdeb41e5")
    assert(header_id != null);
    let tx_id = ergo_wasm.TxId.from_str("258ddfc09b94b8313bca724de44a0d74010cab26de379be845713cc129546b78");
    assert(tx_id != null);

    // Do peer discovery to find active nodes running at least version 4.0.28 (older versions have a
    // bug with nipopow proofs)
    const recent_nodes = [];
    for (let i = 0; i < active_peers.len(); i++) {
        let node_conf = new ergo_wasm.NodeConf(active_peers.get(i));
        let node_info = await ergo_wasm.get_info(node_conf);
        if (node_info.is_at_least_version_4_0_28()) {
            recent_nodes.push(active_peers.get(i));
        }
    }

    assert(recent_nodes.length >= 3, "Expecting at least 3 nodes with version >= v4.0.28");

    // Get NiPoPow proofs from 2 separate ergo nodes
    let proofs = await Promise.all([
        get_nipopow_proof(recent_nodes[0], header_id),
        get_nipopow_proof(recent_nodes[1], header_id),
    ]);

    const genesis_block_id = ergo_wasm.BlockId.from_str("b0244dfc267baca974a4caee06120321562784303a8a688976ae56170e4d175b");
    let verifier = new ergo_wasm.NipopowVerifier(genesis_block_id);
    assert(verifier != null, "verifier should be non-null");
    verifier.process(proofs[0]);
    verifier.process(proofs[1]);
    let best_proof = verifier.best_proof();
    assert(best_proof != null, "best proof should exist");
    assert(best_proof.suffix_head().id().equals(header_id), "equality");

    // Verify with a 3rd node
    let node_conf = new ergo_wasm.NodeConf(recent_nodes[2]);
    let header = await ergo_wasm.get_header(node_conf, header_id);
    assert(header != null, "header should be non-null");
    let merkle_proof = await ergo_wasm.get_blocks_header_id_proof_for_tx_id(node_conf, header_id, tx_id);
    assert(merkle_proof != null, "merkle_proof should be non-null");
    assert(merkle_proof.valid(header.transactions_root()), "merkle_proof should be valid");
});

async function get_nipopow_proof(url, header_id) {
    let node_conf = new ergo_wasm.NodeConf(url);
    assert(node_conf != null);

    // Make sure we're communicating with a node with version >= 4.0.28, otherwise we won't be able
    // to get a proper Nipopow proof
    let node_info = await ergo_wasm.get_info(node_conf);
    assert(node_info.is_at_least_version_4_0_28());

    let proof = await ergo_wasm.get_nipopow_proof_by_header_id(node_conf, 7, 6, header_id);
    assert(proof != null);
    return proof;
}

function get_ergo_node_seeds() {
    return [
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
    ].map(x => new URL(x));
}