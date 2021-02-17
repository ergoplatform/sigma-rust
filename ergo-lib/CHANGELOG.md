# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate
## [0.5.1] - 2021-02-17

### Added 
- Explorer v1 API support for box register parsing [#197](https://github.com/ergoplatform/sigma-rust/pull/197);

## [0.5.0] - 2021-02-04

### Added 
- `CalcBlake2b256` IR node evaluation and serialization [#179](https://github.com/ergoplatform/sigma-rust/pull/179);
- Arith ops (`+, -, *, /`) IR node evaluation and serialization [#181](https://github.com/ergoplatform/sigma-rust/pull/181);
- Comparison ops (`>, >=, <, <=`) IR node evaluation and serialization [#182](https://github.com/ergoplatform/sigma-rust/pull/182);
- `AND`, `Collection` (collection declaration), `BinAnd` IR nodes evaluation and serialization [#183](https://github.com/ergoplatform/sigma-rust/pull/183);
- `Or`, `BinOr` IR nodes evaluation and serialization [#184](https://github.com/ergoplatform/sigma-rust/pull/184);
- `LogicalNot` IR nodes evaluation and serialization [#185](https://github.com/ergoplatform/sigma-rust/pull/185);
- `Map`, `Filter` IR nodes evaluation and serialization [#186](https://github.com/ergoplatform/sigma-rust/pull/186);
- `BoolToSigmaProp`, `If`, `Min`, `Max` IR nodes evaluation and serialization [#187](https://github.com/ergoplatform/sigma-rust/pull/187);
- `ByIndex`, `Box.tokens` IR nodes evaluation and serialization [#188](https://github.com/ergoplatform/sigma-rust/pull/188);

## [0.4.4] - 2021-01-20

### Added 
- `BlockValue`, `ValDef`, `ValUse`, `FuncValue`, `Apply` IR nodes evaluation and serialization [#171](https://github.com/ergoplatform/sigma-rust/pull/171);
- `SimpleBoxSelector`: sort inputs by target tokens and skip inputs that does not have target tokens [#175](https://github.com/ergoplatform/sigma-rust/pull/175);
- `Fold`(collection), `ExtractAmount`(`Box.value`), `SelectField`(tuple field access) IR nodes evaluation and serialization [#173](https://github.com/ergoplatform/sigma-rust/pull/173)

## [0.4.3] - 2021-01-15

### Added 
- `SType::STuple()` and `Value::Tup()` types to store tuples. Implemented serialization, conversion between Rust types and `Constant`(`Value`, `SType`) [#166](https://github.com/ergoplatform/sigma-rust/pull/166);
- `EQ(==)`, `NEQ(!=)` implementation [#166](https://github.com/ergoplatform/sigma-rust/pull/166);

## [0.4.2] - 2020-12-21

### Added 

- Interpreter: Box.Rx properties (get register value), OptionGet [#163](https://github.com/ergoplatform/sigma-rust/pull/163);
- Interpreter: added global vars (`INPUTS`, `OUTPUTS`, `SELF`, `HEIGHT`), `Context` properties (`dataInputs`) [#155](https://github.com/ergoplatform/sigma-rust/pull/155);
- Explorer API v1 format parsing for box.additionalRegisters [#161](https://github.com/ergoplatform/sigma-rust/pull/161);

## [0.4.1] - 2020-11-19

### Added 

- Support for parsing ErgoBox transaction id from `txId` JSON field name;

## [0.4.0] - 2020-11-19

### Added

- Support for parsing ErgoBox id also from "id" JSON field name [#134](https://github.com/ergoplatform/sigma-rust/pull/134)
- Address::p2pk_from_pk_bytes to make an Address from serialized PK [#136](https://github.com/ergoplatform/sigma-rust/pull/136)
- Address::from_str to parse an Address without checking the network prefix [#136](https://github.com/ergoplatform/sigma-rust/pull/136)
- Address::recreate_from_ergo_tree to re-create the address from ErgoTree (built from the address) [#146](https://github.com/ergoplatform/sigma-rust/pull/146)
- NetworkAddress to store Address + NetworkPrefix [#146](https://github.com/ergoplatform/sigma-rust/pull/144)


### Changed

- Move and changed visibility of various modules(input, data_input, prover_result, etc.) [#135](https://github.com/ergoplatform/sigma-rust/pull/135)
- Add Context parameter to Prover::prove, Verifier::verify [#139](https://github.com/ergoplatform/sigma-rust/pull/139)
- Move all transaction-related parameters into TransactionContext parameter in Wallet::sign_transaction [#139](https://github.com/ergoplatform/sigma-rust/pull/139)
- Move Constant export from crate root to constant module (ast::constant) and made eval module private [#142](https://github.com/ergoplatform/sigma-rust/pull/142)
- Make SType public [#142](https://github.com/ergoplatform/sigma-rust/pull/142)

## [0.3.0] - 2020-11-04

### Added

- Add value extraction API for Constant (e.g i64::try_extract_from(constant))  [#111](https://github.com/ergoplatform/sigma-rust/pull/111).
- Implement From<BoxId> for DataInput [#113](https://github.com/ergoplatform/sigma-rust/pull/113).
- Add data inputs to TxBuilder [#115](https://github.com/ergoplatform/sigma-rust/pull/115).
- Read/Write register values in ErgoBox, ErgoBoxCandidate [#116](https://github.com/ergoplatform/sigma-rust/pull/116).
- Add tokens support in TxBuilder and ErgoBoxCandidateBuilder [#118](https://github.com/ergoplatform/sigma-rust/pull/118).
- Implement JSON encoding/decoding for UnsignedTransaction [#123](https://github.com/ergoplatform/sigma-rust/pull/123).
- Add TxBuilder::estimate_tx_size_bytes() to get estimated serialized transaction size in bytes after signing (assuming P2PK box spending); tx_builder::SUGGESTED_TX_FEE constant with "default" current tx fee used lately (1100000 nanoERGs); [#128](https://github.com/ergoplatform/sigma-rust/pull/128).
- Add checks when minting token for minting token exclusivity and registers overwrite [#129](https://github.com/ergoplatform/sigma-rust/pull/129).
- Add transaction validity checks in TxBuilder [#130](https://github.com/ergoplatform/sigma-rust/pull/130).
- Use TokenAmount instead of u64 in sum_tokens*() [#130](https://github.com/ergoplatform/sigma-rust/pull/130).
- Add TokenAmount::checked*() ops [#130](https://github.com/ergoplatform/sigma-rust/pull/130).
- Add I64::as_num() in WASM bindings [#130](https://github.com/ergoplatform/sigma-rust/pull/130)
 

### Changed

- box_id, box_value and register modules made private in ergo_box module and all types are re-exported from ergo_box module itself [#131](https://github.com/ergoplatform/sigma-rust/pull/131).


## [0.2.0] - 2020-10-06

### Added

- Binary serialization;
- JSON serialization;
- Box, Transaction building;
- Transaction signing (P2PK only);
- ErgoTree constant values conversion.

<!-- next-url -->
[Unreleased]: https://github.com/ergoplatform/sigma-rust/compare/ergo-lib-v0.5.1...HEAD
[0.5.1]: https://github.com/ergoplatform/sigma-rust/compare/ergo-lib-v0.5.0...ergo-lib-v0.5.1
[0.5.0]: https://github.com/ergoplatform/sigma-rust/compare/ergo-lib-v0.4.4...ergo-lib-v0.5.0
[0.4.4]: https://github.com/ergoplatform/sigma-rust/compare/ergo-lib-v0.4.3...ergo-lib-v0.4.4
[0.4.3]: https://github.com/ergoplatform/sigma-rust/compare/ergo-lib-v0.4.2...ergo-lib-v0.4.3
[0.4.2]: https://github.com/ergoplatform/sigma-rust/compare/ergo-lib-v0.4.1...ergo-lib-v0.4.2
[0.4.1]: https://github.com/ergoplatform/sigma-rust/compare/ergo-lib-v0.4.0...ergo-lib-v0.4.1
[0.4.0]: https://github.com/ergoplatform/sigma-rust/compare/ergo-lib-v0.3.0...ergo-lib-v0.4.0
[0.3.0]: https://github.com/ergoplatform/sigma-rust/compare/v0.2.0...ergo-lib-v0.3.0
[0.2.0]: https://github.com/ergoplatform/sigma-rust/compare/v0.1.0...v0.2.0        
