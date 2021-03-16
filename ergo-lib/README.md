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

[ErgoTree Specification](https://github.com/ScorexFoundation/sigmastate-interpreter/tree/develop/docs/spec)

## ErgoTree interpreter

Implemented operations (IR nodes) are denoted with :heavy_check_mark:.
Descriptions for the operations can be found in [ErgoTree Specification](https://github.com/ScorexFoundation/sigmastate-interpreter/tree/develop/docs/spec)

### General

- :heavy_check_mark: Blocks (`BlockValue`);
- :heavy_check_mark: Variable definition (`ValDef`, `ValUse`);
- :heavy_check_mark: Function definition (`FuncValue`);
- :heavy_check_mark: Function application(`Apply`);
- :heavy_check_mark: Tuplse field access
- :heavy_check_mark: 'If' conditional

### Operations

- :heavy_check_mark: comparison: `>, <, >=, <=, ==, !=`;
- :heavy_check_mark: arithmetic: ` +, -, *, /, %`;
- :heavy_check_mark: logical: ` &&, ||`;
- | (bitwise OR);
- & (bitwise AND);
- ^ (logical XOR);
- ^ (bitwise XOR);
- `|` (byte-wise XOR of two collections of bytes);
- :heavy_check_mark: unary `!`;
- unary `-`;
- unary `~` (bit inversion);
- `>>`, `<<`, `>>>` (bit shifts);

### Predefined functions

- groupGenerator
- xor
- substConstants
- longToByteArray
- byteArrayToBigInt
- byteArrayToLong
- downcast
- :heavy_check_mark: upcast
- :heavy_check_mark: allOf
- :heavy_check_mark: anyOf
- atLeast
- :heavy_check_mark: min
- :heavy_check_mark: max
- avlTree
- treeLookup
- :heavy_check_mark: blake2b256
- sha256
- :heavy_check_mark: proveDlog
- proveDHTuple
- sigmaProp
- executeFromVar
- executeFromSelfReg
- getVar
- allZK
- anyZK
- decodePoint
- xorOf

### Object properties and methods

#### Byte

- toByte
- toShort
- toInt
- toLong
- toBigInt
- toBytes
- toBits

#### Short

- toByte
- toShort
- toInt
- toLong
- toBigInt
- toBytes
- toBits

#### Int

- toByte
- toShort
- toInt
- toLong
- toBigInt
- toBytes
- toBits

#### Long

- toByte
- toShort
- toInt
- toLong
- toBigInt
- toBytes
- toBits

#### BigInt

- toByte
- toShort
- toInt
- toLong
- toBigInt
- toBytes
- toBits

#### GroupElement

- getEncoded
- exp
- multiply
- negate

#### SigmaProp

- :heavy_check_mark: propBytes
- isProven

#### Box

- :heavy_check_mark: value
- :heavy_check_mark: propositionBytes
- bytes
- bytesWithoutRef
- :heavy_check_mark: id
- :heavy_check_mark: creationInfo
- :heavy_check_mark: getReg
- :heavy_check_mark: tokens
- :heavy_check_mark: R0 .. R9

#### AvlTree 

- digest
- enabledOperations
- keyLength
- valueLengthOpt
- isInsertAllowed
- isUpdateAllowed
- isRemoveAllowed
- updateOperations
- contains
- get
- getMany
- insert
- update
- remove
- updateDigest

#### Header

- id
- version
- parentId
- ADProofsRoot
- stateRoot
- transactionsRoot
- timestamp
- nBits
- height
- extensionRoot
- minerPk
- powOnetimePk
- powNonce
- powDistance
- votes


#### PreHeader

- version
- parentId
- timestamp
- nBits
- height
- minerPk
- votes


#### Context

- :heavy_check_mark: dataInputs
- headers
- preHeader
- :heavy_check_mark: INPUTS
- :heavy_check_mark: OUTPUTS
- :heavy_check_mark: HEIGHT
- :heavy_check_mark: SELF
- selfBoxIndex
- LastBlockUtxoRootHash
- minerPubKey
- getVar

#### Collection

- :heavy_check_mark: size
- getOrElse
- :heavy_check_mark: map
- :heavy_check_mark: exists
- :heavy_check_mark: fold
- forall
- slice
- :heavy_check_mark: filter
- append
- :heavy_check_mark: apply
- indices
- flatMap
- patch
- updated
- updateMany
- indexOf
- zip

#### Option

- :heavy_check_mark: isDefined
- :heavy_check_mark: get
- :heavy_check_mark: getOrElse
- map
- filter


## Crate features
### `json` (default feature)
JSON serialization for chain types using `serde`.

### `compiler` (default feature)
Compile `ErgoTree` from ErgoScript via `Contract::compile`.
