// These integration tests assume that a local ergo node instance is running and its REST API is
// accessible @ 127.0.0.1:9053.

import { assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
    ergo_wasm = await ergo;
});

it('node REST API get_info endpoint', async () => {
    let node_conf = new ergo_wasm.NodeConf(new URL("http://127.0.0.1:9053"));
    assert(node_conf != null);
    let res = await ergo_wasm.get_info(node_conf);
    assert(res != null);
    assert(node_conf != null);
});