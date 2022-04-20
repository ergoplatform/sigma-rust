
import { expect, assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
    ergo_wasm = await ergo;
});

it("Message signing smoke test", async () => {
    const sk = ergo_wasm.SecretKey.random_dlog();
    const addr = sk.get_address()
    const sks = new ergo_wasm.SecretKeys();
    assert(addr != null);
    sks.add(sk);
    const wallet = ergo_wasm.Wallet.from_secrets(sks);
    const message = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
    const signature = wallet.sign_message_using_p2pk(addr, message);
    assert(signature != null);
    const res = ergo_wasm.verify_signature(addr, message, signature);
    assert(res == true);
});