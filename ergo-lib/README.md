[![Latest Version](https://img.shields.io/crates/v/ergo-lib.svg)](https://crates.io/crates/ergo-lib)
[![Documentation](https://docs.rs/ergo-lib/badge.svg)](https://docs.rs/crate/ergo-lib)

## Features
### Implemented:
- Binary serialization;
- JSON serialization;
- `ErgoTree` evaluation;
- Box builder(with mint token support);
- Transaction building and signing;
- Box selection for funds and assets (with token burning support);
- Box registers(R4-R9) access;

### Roadmap:
- Interpreter:
    - CThreshold conjecture in sigma protocol; [#248](https://github.com/ergoplatform/sigma-rust/issues/248)
    - AVL trees; [#368](https://github.com/ergoplatform/sigma-rust/issues/368)
    - ~~`Context.headers`; [#372](https://github.com/ergoplatform/sigma-rust/issues/372)~~
    - ~~`Context.preHeader`; [#373](https://github.com/ergoplatform/sigma-rust/issues/373)~~
    - Distributed signatures; [#367](https://github.com/ergoplatform/sigma-rust/issues/367)
    - ~~Rest of the unimplemented ops (a few not-so-popular ones, see [`ergotree-ir` README](../ergotree-ir/README.md));~~
    - Costing; [#193](https://github.com/ergoplatform/sigma-rust/issues/193)
- Swift bindings for iOS; [#47](https://github.com/ergoplatform/sigma-rust/issues/47)
- ErgoScript compiler; [#370](https://github.com/ergoplatform/sigma-rust/issues/370)
- `ErgoTree` pretty printer ("decompiler"); [#371](https://github.com/ergoplatform/sigma-rust/issues/371)
- Kotlin bindings for Android; [#369](https://github.com/ergoplatform/sigma-rust/issues/369)

Bindings:
- [ergo-lib-wasm](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-wasm) [![Latest Version](https://img.shields.io/crates/v/ergo-lib-wasm.svg)](https://crates.io/crates/ergo-lib-wasm) [![Documentation](https://docs.rs/ergo-lib-wasm/badge.svg)](https://docs.rs/crate/ergo-lib-wasm) 
- [ergo-lib-wasm-browser](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-wasm) [![Latest version](https://img.shields.io/npm/v/ergo-lib-wasm-browser)](https://www.npmjs.com/package/ergo-lib-wasm-browser)
- [ergo-lib-wasm-nodejs](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-wasm) [![Latest version](https://img.shields.io/npm/v/ergo-lib-wasm-nodejs)](https://www.npmjs.com/package/ergo-lib-wasm-nodejs)
- [ergo-lib-ios](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-ios)
- [ergo-lib-jni](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-jni) [![Latest Version](https://img.shields.io/crates/v/ergo-lib-jni.svg)](https://crates.io/crates/ergo-lib-jni) [![Documentation](https://docs.rs/ergo-lib-jni/badge.svg)](https://docs.rs/crate/ergo-lib-jni)
- [ergo-lib-c](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-c) [![Latest Version](https://img.shields.io/crates/v/ergo-lib-c.svg)](https://crates.io/crates/ergo-lib-c) [![Documentation](https://docs.rs/ergo-lib-c/badge.svg)](https://docs.rs/crate/ergo-lib-c)

## ErgoScript Language

[ErgoScript Language Description](https://github.com/ScorexFoundation/sigmastate-interpreter/blob/develop/docs/LangSpec.md)

## Crate features
### `json` (default feature)
JSON serialization for chain types using `serde`.

### `compiler` (default feature)
Compile `ErgoTree` from ErgoScript via `Contract::compile`.

