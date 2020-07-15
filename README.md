[![Build Status](https://travis-ci.com/ergoplatform/sigma-rust.svg?branch=develop)](https://travis-ci.com/ergoplatform/sigma-rust)
[![Coverage Status](https://coveralls.io/repos/github/ergoplatform/sigma-rust/badge.svg)](https://coveralls.io/github/ergoplatform/sigma-rust)

Rust implementation of [ErgoScript](https://github.com/ScorexFoundation/sigmastate-interpreter) cryptocurrency scripting language. 

## Crates
[sigma-tree](https://github.com/ergoplatform/sigma-rust/tree/develop/sigma-tree) - ErgoTree, interpreter, chain types (transactions, boxes, etc.), JSON serialization.

[ergo-wallet](https://github.com/ergoplatform/sigma-rust/tree/develop/ergo-wallet) - Transaction creation and signing. Has bindings for the following platforms:
- [JS/TS(WASM)](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-wallet-wasm)
- [iOS](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-wallet-ios)
- [C](https://github.com/ergoplatform/sigma-rust/tree/develop/bindings/ergo-wallet-c)

## Contributing
Contributions are very welcome! Checkout issues labeled ["help wanted" and "good first issue"](https://github.com/ergoplatform/sigma-rust/labels/help%20wanted)
We check code formatting and linter(`clippy`) as part of the CI process. It's better to set up your editor to run `rustfmt` on file save.
 Feel free to disable specific linter rules in the source code when appropriate.
