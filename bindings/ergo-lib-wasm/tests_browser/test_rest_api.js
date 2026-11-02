import { expect, assert, AssertionError } from "chai";

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
    ergo_wasm = await ergo;
});

it('node REST helpers: ordered fallback tries each distinct host once', async () => {
    const first = new URL("http://127.0.0.1:19053/first");
    const same_endpoint = new URL("https://127.0.0.1:19053/second");
    const second = new URL("http://127.0.0.1:29053");
    const attempts = [];

    const result = await with_fallback_node((_node_conf, url) => {
        attempts.push(url.href);
        if (url.href === first.href) {
            throw fixture_node_error("reqwest error: error sending request");
        }
        return "success";
    }, [first, same_endpoint, second]);

    expect(result).to.equal("success");
    expect(attempts).to.deep.equal([first.href, second.href]);
});

it('node REST helpers: all-fail fallback reports every attempted URL and final cause', async () => {
    const urls = [
        new URL("http://127.0.0.1:19053"),
        new URL("http://127.0.0.1:29053"),
    ];

    let failure;
    try {
        await with_fallback_node((_node_conf, url) => {
            throw fixture_node_error("reqwest error: error sending request");
        }, urls);
    } catch (e) {
        failure = e;
    }

    expect(failure).to.be.an("error");
    expect(failure.name).to.equal("NodeFallbackError");
    expect(failure.attemptedUrls).to.deep.equal(urls.map(url => url.href));
    expect(failure.message).to.include(urls[0].href);
    expect(failure.message).to.include(urls[1].href);
    expect(failure.message).to.include("reqwest error: error sending request");
});

it('node REST helpers: empty fallback reports that no URL was attempted', async () => {
    let failure;
    try {
        await with_fallback_node(() => "unused", []);
    } catch (e) {
        failure = e;
    }

    expect(failure).to.be.an("error");
    expect(failure.name).to.equal("NodeFallbackError");
    expect(failure.attemptedUrls).to.deep.equal([]);
    expect(failure.message).to.include("attempted URLs: none");
});

it('node REST helpers: proof collection skips one failed URL then returns two distinct proofs', async () => {
    const urls = [
        new URL("http://127.0.0.1:19053"),
        new URL("http://127.0.0.1:29053"),
        new URL("http://127.0.0.1:39053"),
    ];
    const attempts = [];

    const proofs = await collect_two_nipopow_proofs(urls, async url => {
        attempts.push(url.href);
        if (url.href === urls[0].href) {
            throw fixture_node_error("reqwest error: error sending request");
        }
        return `proof from ${url.href}`;
    });

    expect(proofs).to.deep.equal([
        `proof from ${urls[1].href}`,
        `proof from ${urls[2].href}`,
    ]);
    expect(attempts).to.deep.equal(urls.map(url => url.href));
});

it('node REST helpers: URLs with the same effective host cannot satisfy the two-source precondition', async () => {
    const first = new URL("http://127.0.0.1:19053/first");
    const same_endpoint = new URL("https://127.0.0.1:19053/second");
    let calls = 0;
    let failure;

    try {
        await collect_two_nipopow_proofs([first, same_endpoint], async () => {
            calls += 1;
            return "one proof";
        });
    } catch (e) {
        failure = e;
    }

    expect(calls).to.equal(1);
    expect(failure).to.be.an("error");
    expect(failure.name).to.equal("InsufficientNipopowSourcesError");
    expect(failure.attemptedUrls).to.deep.equal([first.href]);
    expect(failure.successfulUrls).to.deep.equal([first.href]);
    expect(is_external_node_unavailability(failure)).to.equal(false);
});

it('node REST helpers: one proof plus one unavailable source remains insufficient', async () => {
    const urls = [
        new URL("http://127.0.0.1:19053"),
        new URL("http://127.0.0.1:29053"),
    ];
    let failure;

    try {
        await collect_two_nipopow_proofs(urls, async url => {
            if (url.href === urls[0].href) {
                return "one proof";
            }
            throw fixture_node_error("reqwest error: error sending request");
        });
    } catch (e) {
        failure = e;
    }

    expect(failure).to.be.an("error");
    expect(failure.name).to.equal("InsufficientNipopowSourcesError");
    expect(failure.successfulUrls).to.deep.equal([urls[0].href]);
    expect(is_external_node_unavailability(failure)).to.equal(true);
});

it('node REST helpers: only the REST binding sending-request error classifies as external', () => {
    expect(is_recognizable_external_node_error(
        fixture_node_error("reqwest error: error sending request")
    )).to.equal(true);
    expect(is_recognizable_external_node_error(
        fixture_node_error("reqwest error: error decoding response body")
    )).to.equal(false);
    expect(is_recognizable_external_node_error(new TypeError("Failed to fetch"))).to.equal(false);
    expect(is_recognizable_external_node_error(new Error("connection refused"))).to.equal(false);
    expect(is_recognizable_external_node_error(new Error("request timed out"))).to.equal(false);

    const assertion_failure = new Error("expected timeout assertion to hold");
    assertion_failure.name = "AssertionError";
    expect(is_recognizable_external_node_error(assertion_failure)).to.equal(false);
});

it('node REST helpers: fallback rethrows local errors unchanged without trying another host', async () => {
    const urls = [
        new URL("http://127.0.0.1:19053"),
        new URL("http://127.0.0.1:29053"),
    ];
    const local_errors = [
        fixture_node_error("reqwest error: error decoding response body"),
        new AssertionError("local assertion failure"),
    ];

    for (const expected_error of local_errors) {
        const attempts = [];
        let failure;
        try {
            await with_fallback_node((_node_conf, url) => {
                attempts.push(url.href);
                throw expected_error;
            }, urls);
        } catch (e) {
            failure = e;
        }

        expect(failure).to.equal(expected_error);
        expect(attempts).to.deep.equal([urls[0].href]);
    }
});

it('node REST helpers: proof collection rethrows local errors unchanged without trying another host', async () => {
    const urls = [
        new URL("http://127.0.0.1:19053"),
        new URL("http://127.0.0.1:29053"),
    ];
    const local_errors = [
        fixture_node_error("reqwest error: error decoding response body"),
        new AssertionError("local assertion failure"),
    ];

    for (const expected_error of local_errors) {
        const attempts = [];
        let failure;
        try {
            await collect_two_nipopow_proofs(urls, async url => {
                attempts.push(url.href);
                throw expected_error;
            });
        } catch (e) {
            failure = e;
        }

        expect(failure).to.equal(expected_error);
        expect(attempts).to.deep.equal([urls[0].href]);
    }
});

// Note that the REST API tests are here due to the WASM implementation of `reqwest-wrap`. In
// particular the timeout functionality for HTTP requests requires the window object from the
// web APIs, thus requiring a web browser to run.

it('node REST API: peer_discovery endpoint', async () => {
    const seeds = get_ergo_node_seeds();
    // Limit to 150 simultaneous HTTP requests and search for peers for 140 seconds (remember
    // there's an unavoidable waiting time of 80 seconds, to give Chrome time to relinquish failed
    // preflight requests)
    let is_chrome = true;
    let active_peers = await ergo_wasm.peer_discovery(seeds, 20, 200, is_chrome);
    assert(active_peers.len() > 0);
    console.log("Number active peers:", active_peers.len(), ". First active peer: ", active_peers.get(0).href);
});

it('node REST API: peer_discovery endpoint (INCREMENTAL VERSION)', async () => {
    const seeds = get_ergo_node_seeds();
    let scan = new ergo_wasm.ChromePeerDiscoveryScan(seeds);

    scan = await ergo_wasm.incremental_peer_discovery_chrome(scan, 20, 200);
    let scan_1_len = scan.active_peers().len();
    console.log("# active peers from first scan:", scan_1_len);
    scan = await ergo_wasm.incremental_peer_discovery_chrome(scan, 20, 480);
    let scan_2_len = scan.active_peers().len();
    console.log("# active peers from second scan:", scan_2_len);

    // The following assert should have `<` instead of `<=`. There is an issue with Github CI, see
    // https://github.com/ergoplatform/sigma-rust/issues/586
    assert(scan_1_len <= scan_2_len, "Should have found more peers after second scan!");
});

// Known-live mainnet nodes (REST API on :9053) used as fallbacks for network-dependent tests.
const MAINNET_NODE_URLS = [
    "http://213.239.193.208:9053",
    "http://159.65.11.55:9053",
].map(x => new URL(x));

it('node REST API: get_nipopow_proof_by_header_id endpoint', async () => {
    const header_id = ergo_wasm.BlockId.from_str("4caa17e62fe66ba7bd69597afdc996ae35b1ff12e0ba90c22ff288a4de10e91b");
    let res = await with_fallback_node(node_conf =>
        ergo_wasm.get_nipopow_proof_by_header_id(node_conf, 3, 4, header_id));
    assert(res != null);
});

it('node REST API: example SPV workflow', async function () {
    try {
        const header_id = ergo_wasm.BlockId.from_str("d1366f762e46b7885496aaab0c42ec2950b0422d48aec3b91f45d4d0cdeb41e5")
        assert(header_id != null);
        let tx_id = ergo_wasm.TxId.from_str("258ddfc09b94b8313bca724de44a0d74010cab26de379be845713cc129546b78");
        assert(tx_id != null);

        const proofs = await collect_two_nipopow_proofs(
            MAINNET_NODE_URLS,
            url => get_nipopow_proof(url, header_id),
        );
        assert.strictEqual(proofs.length, 2, "SPV workflow requires two distinct proof sources");

        const genesis_block_id = ergo_wasm.BlockId.from_str("b0244dfc267baca974a4caee06120321562784303a8a688976ae56170e4d175b");
        let verifier = new ergo_wasm.NipopowVerifier(genesis_block_id);
        assert(verifier != null, "verifier should be non-null");
        for (const proof of proofs) {
            verifier.process(proof);
        }
        let best_proof = verifier.best_proof();
        assert(best_proof != null, "best proof should exist");
        assert(best_proof.suffix_head().id().equals(header_id), "equality");

        // Verify against a reachable node
        let header = await with_fallback_node(node_conf => ergo_wasm.get_header(node_conf, header_id));
        assert(header != null, "header should be non-null");
        let merkle_proof = await with_fallback_node(node_conf =>
            ergo_wasm.get_blocks_header_id_proof_for_tx_id(node_conf, header_id, tx_id));
        assert(merkle_proof != null, "merkle_proof should be non-null");
        assert(merkle_proof.valid(header.transactions_root()), "merkle_proof should be valid");
    } catch (e) {
        if (is_external_node_unavailability(e)) {
            console.warn("Skipping SPV workflow because public nodes are unavailable:", e.message);
            this.skip();
            return;
        }
        throw e;
    }
});

// Run `fn` against each distinct node URL, returning the first success.
async function with_fallback_node(fn, urls = MAINNET_NODE_URLS) {
    const attempted_urls = [];
    const failures = [];
    for (const url of distinct_urls(urls)) {
        attempted_urls.push(url.href);
        try {
            const node_conf = new ergo_wasm.NodeConf(url);
            return await fn(node_conf, url);
        } catch (e) {
            console.log("node request failed for", url.href, e);
            if (!is_recognizable_external_node_error(e)) {
                throw e;
            }
            failures.push({ url: url.href, cause: e });
        }
    }

    const final_cause = failures.length > 0
        ? describe_error(failures[failures.length - 1].cause)
        : "none";
    const error = new Error(
        `node fallback failed; attempted URLs: ${format_urls(attempted_urls)}; final cause: ${final_cause}`
    );
    error.name = "NodeFallbackError";
    error.attemptedUrls = attempted_urls;
    error.causes = failures.map(failure => ({
        url: failure.url,
        cause: describe_error(failure.cause),
    }));
    error.externalFailuresOnly = failures.length > 0;
    throw error;
}

async function get_nipopow_proof(url, header_id) {
    let node_conf = new ergo_wasm.NodeConf(url);
    assert(node_conf != null);

    // Make sure we're communicating with a node with version >= 4.0.100, due to the EIP-37 hard-fork.
    let node_info = await ergo_wasm.get_info(node_conf);
    assert(node_info.is_at_least_version_4_0_100(), "Ergo node should be at least version 4.0.100");

    let proof = await ergo_wasm.get_nipopow_proof_by_header_id(node_conf, 7, 6, header_id);
    assert(proof != null);
    return proof;
}

async function collect_two_nipopow_proofs(urls, fetch_proof) {
    const attempted_urls = [];
    const successful_urls = [];
    const proofs = [];
    const failures = [];

    for (const url of distinct_urls(urls)) {
        attempted_urls.push(url.href);
        try {
            const proof = await fetch_proof(url);
            proofs.push(proof);
            successful_urls.push(url.href);
            if (proofs.length === 2) {
                return proofs;
            }
        } catch (e) {
            console.log("get_nipopow_proof failed for", url.href, e);
            if (!is_recognizable_external_node_error(e)) {
                throw e;
            }
            failures.push({ url: url.href, cause: e });
        }
    }

    const causes = failures.length > 0
        ? failures.map(failure => `${failure.url}: ${describe_error(failure.cause)}`).join("; ")
        : "none";
    const error = new Error(
        `insufficient distinct NiPoPoW sources: ${proofs.length} success(es); ` +
        `attempted URLs: ${format_urls(attempted_urls)}; causes: ${causes}`
    );
    error.name = "InsufficientNipopowSourcesError";
    error.attemptedUrls = attempted_urls;
    error.successfulUrls = successful_urls;
    error.causes = failures.map(failure => ({
        url: failure.url,
        cause: describe_error(failure.cause),
    }));
    error.externalFailuresOnly = failures.length > 0;
    throw error;
}

function distinct_urls(urls) {
    const seen = new Set();
    return urls.filter(url => {
        // NodeConf retains only URL.host, so scheme and path do not identify a new node.
        if (seen.has(url.host)) {
            return false;
        }
        seen.add(url.host);
        return true;
    });
}

function format_urls(urls) {
    return urls.length > 0 ? urls.join(", ") : "none";
}

function describe_error(error) {
    if (error instanceof Error) {
        return `${error.name}: ${error.message}`;
    }
    return String(error);
}

function is_recognizable_external_node_error(error) {
    return error instanceof Error &&
        error.name === "NodeError" &&
        error.message === "reqwest error: error sending request";
}

function is_external_node_unavailability(error) {
    return error instanceof Error &&
        (error.name === "NodeFallbackError" ||
            error.name === "InsufficientNipopowSourcesError") &&
        error.externalFailuresOnly === true;
}

function fixture_node_error(message) {
    const error = new Error(message);
    error.name = "NodeError";
    return error;
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
