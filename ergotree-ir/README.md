[![Latest Version](https://img.shields.io/crates/v/ergotree-ir.svg)](https://crates.io/crates/ergotree-ir)
[![Documentation](https://docs.rs/ergotree-ir/badge.svg)](https://docs.rs/crate/ergotree-ir)

## Features:
- ErgoTree types, values, IR nodes definition;
- ErgoTree IR nodes serialization;

## ErgoTree IR

[ErgoTree Specification](https://github.com/ScorexFoundation/sigmastate-interpreter/tree/develop/docs/spec)

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
- :heavy_check_mark: | (bitwise OR);
- :heavy_check_mark: & (bitwise AND);
- :heavy_check_mark: ^ (bitwise XOR);
- ^ (logical XOR);
- `|` (byte-wise XOR of two collections of bytes);
- :heavy_check_mark: unary `!`;
- :heavy_check_mark: unary `-`;
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
- :heavy_check_mark: proveDHTuple
- :heavy_check_mark: sigmaProp
- executeFromVar
- executeFromSelfReg
- getVar
- :heavy_check_mark: allZK
- :heavy_check_mark: anyZK
- :heavy_check_mark: decodePoint
- xorOf

### Object properties and methods

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
- :heavy_check_mark: minerPubKey
- :heavy_check_mark: getVar

#### Collection

- :heavy_check_mark: size
- :heavy_check_mark: getOrElse
- :heavy_check_mark: map
- :heavy_check_mark: exists
- :heavy_check_mark: fold
- :heavy_check_mark: forall
- slice
- :heavy_check_mark: filter
- append
- :heavy_check_mark: apply
- indices
- :heavy_check_mark: flatMap
- patch
- updated
- updateMany
- :heavy_check_mark: indexOf
- zip

#### Option

- :heavy_check_mark: isDefined
- :heavy_check_mark: get
- :heavy_check_mark: getOrElse
- map
- filter

