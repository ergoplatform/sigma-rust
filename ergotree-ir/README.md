[![Latest Version](https://img.shields.io/crates/v/ergotree-ir.svg)](https://crates.io/crates/ergotree-ir)
[![Documentation](https://docs.rs/ergotree-ir/badge.svg)](https://docs.rs/crate/ergotree-ir)

## Features:
- ErgoTree types, values, IR nodes definition;
- ErgoTree IR nodes serialization;

## ErgoTree IR

[ErgoTree Specification](https://github.com/ScorexFoundation/sigmastate-interpreter/tree/develop/docs/spec)

Descriptions for the operations can be found in [ErgoTree Specification](https://github.com/ScorexFoundation/sigmastate-interpreter/tree/develop/docs/spec)

# Not yet implemented operations:

### Object properties and methods

#### SigmaProp

- isProven

## Implemented operations (IR nodes): 

### General

- Blocks (`BlockValue`);
- Variable definition (`ValDef`, `ValUse`);
- Function definition (`FuncValue`);
- Function application(`Apply`);
- Tuplse field access
- 'If' conditional

### Operations

- comparison: `>, <, >=, <=, ==, !=`;
- arithmetic: ` +, -, *, /, %`;
- logical: ` &&, ||`;
- | (bitwise OR);
- & (bitwise AND);
- ^ (bitwise XOR);
- `|` (byte-wise XOR of two collections of bytes) [#296](https://github.com/ergoplatform/sigma-rust/issues/296);
- unary `!`;
- unary `-`;
- ^ (logical XOR);
- unary `~` (bit inversion);

### Predefined functions

- longToByteArray
- byteArrayToBigInt
- byteArrayToLong
- upcast
- allOf
- anyOf
- min
- max
- blake2b256
- sha256
- proveDlog
- proveDHTuple
- sigmaProp
- substConstants
- executeFromVar
- executeFromSelfReg
- getVar
- allZK
- anyZK
- decodePoint
- groupGenerator [#332](https://github.com/ergoplatform/sigma-rust/issues/332)
- xorOf [#356](https://github.com/ergoplatform/sigma-rust/issues/356)
- downcast
- avlTree
- treeLookup
- atLeast

### Object properties and methods

#### GroupElement

- exp [#297](https://github.com/ergoplatform/sigma-rust/issues/297)
- multiply [#298](https://github.com/ergoplatform/sigma-rust/issues/298)
- getEncoded [#330](https://github.com/ergoplatform/sigma-rust/issues/330)
- negate [#331](https://github.com/ergoplatform/sigma-rust/issues/331)

#### SigmaProp

- propBytes

#### Box

- value
- propositionBytes
- id
- creationInfo
- getReg
- tokens
- R0 .. R9
- bytes
- bytesWithoutRef

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

- dataInputs
- INPUTS
- OUTPUTS
- HEIGHT
- SELF
- minerPubKey
- getVar
- selfBoxIndex
- headers
- preHeader
- LastBlockUtxoRootHash

#### AvlTree 

- digest
- enabledOperations
- keyLength
- valueLengthOpt
- isInsertAllowed
- isUpdateAllowed
- isRemoveAllowed
- updateOperations
- insert
- updateDigest
- contains
- get
- getMany
- update
- remove

#### Collection

- size
- getOrElse
- map
- exists
- fold
- forall
- slice [#300](https://github.com/ergoplatform/sigma-rust/issues/300)
- filter
- append [#301](https://github.com/ergoplatform/sigma-rust/issues/301)
- apply
- flatMap
- indexOf
- zip [#329](https://github.com/ergoplatform/sigma-rust/issues/329)
- indices [#314](https://github.com/ergoplatform/sigma-rust/issues/314)
- patch [#357](https://github.com/ergoplatform/sigma-rust/issues/357)
- updated [#358](https://github.com/ergoplatform/sigma-rust/issues/358)
- updateMany [#359](https://github.com/ergoplatform/sigma-rust/issues/359)

#### Option

- isDefined
- get
- getOrElse
- map [#360](https://github.com/ergoplatform/sigma-rust/issues/360)
- filter [#360](https://github.com/ergoplatform/sigma-rust/issues/360)

