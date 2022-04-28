import { expect, assert } from "chai";
import { generate_block_headers } from '../tests/utils';

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
    ergo_wasm = await ergo;
});

// Note that the REST API tests are here due to the WASM implementation of `reqwest-wrap`. In
// particular the timeout functionality for HTTP requests requires the window object from the
// web APIs, thus requiring a web browser to run.

it('node REST API: get_nipopow_proof_by_header_id endpoint', async () => {
    let node_conf = new ergo_wasm.NodeConf("213.239.193.208:9053");
    assert(node_conf != null);
    const block_headers = generate_block_headers();
    const header_id = block_headers.get(0).id();
    let res = await ergo_wasm.get_nipopow_proof_by_header_id(node_conf, 3, 4, header_id);
    assert(res != null);
    assert(node_conf != null);
});

it('node REST API: peer_discovery endpoint', async () => {
    const seeds = [
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
    let res = await ergo_wasm.peer_discovery(seeds, 10, 3);
    assert(res.len() > 0, "Should be at least one peer!");
});
