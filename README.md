[![Build Status](https://travis-ci.com/ergoplatform/sigma-rust.svg?branch=develop)](https://travis-ci.com/ergoplatform/sigma-rust)
[![Coverage Status](https://coveralls.io/repos/github/ergoplatform/sigma-rust/badge.svg)](https://coveralls.io/github/ergoplatform/sigma-rust)

Rust implementation of [ErgoScript](https://github.com/ScorexFoundation/sigmastate-interpreter) cryptocurrency scripting language. 

## Crates
[ergo-lib](https://github.com/ergoplatform/sigma-rust/tree/develop/ergo-lib) - ErgoTree, interpreter, chain types (transactions, boxes, etc.), JSON serialization, tx creation and signing.

Has bindings for the following platforms:
- [JS/TS(WASM)](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-wasm)
- [iOS](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-ios)
- [Android(JNI)](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-android)
- [C](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-lib-c)

## Contributing
Contributions are very welcome! Checkout issues labeled ["help wanted" and "good first issue"](https://github.com/ergoplatform/sigma-rust/labels/help%20wanted)
We check code formatting and linter(`clippy`) as part of the CI process. It's better to set up your editor to run `rustfmt` on file save.
 Feel free to disable specific linter rules in the source code when appropriate.
