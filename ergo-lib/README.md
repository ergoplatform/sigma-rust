[![Latest Version](https://img.shields.io/crates/v/ergo-lib.svg)](https://crates.io/crates/ergo-lib)
[![Documentation](https://docs.rs/ergo-lib/badge.svg)](https://docs.rs/crate/ergo-lib)

## Features:
- ErgoTree AST;
- Transactions, boxes, etc.;
- JSON serialization;
- Box builder(with mint token support);
- Transaction creation(builder) and signing;
- Box selection for funds and assets (with token burning support);

## ErgoScript Language

[ErgoScript Language Description](https://github.com/ScorexFoundation/sigmastate-interpreter/blob/develop/docs/LangSpec.md)

## ErgoTree interpreter

To check what IR nodes are implemented check out [ergotree-ir README](../ergotree-ir/README.md).


## Crate features
### `json` (default feature)
JSON serialization for chain types using `serde`.

### `compiler` (default feature)
Compile `ErgoTree` from ErgoScript via `Contract::compile`.
