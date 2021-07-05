[![Latest Version](https://img.shields.io/crates/v/ergotree-ir.svg)](https://crates.io/crates/ergotree-ir)
[![Documentation](https://docs.rs/ergotree-ir/badge.svg)](https://docs.rs/crate/ergotree-ir)

## Features:
- ErgoTree types, values, IR nodes definition;
- ErgoTree IR nodes serialization;

## ErgoTree IR

[ErgoTree Specification](https://github.com/ScorexFoundation/sigmastate-interpreter/tree/develop/docs/spec)

Descriptions for the operations can be found in [ErgoTree Specification](https://github.com/ScorexFoundation/sigmastate-interpreter/tree/develop/docs/spec)

Not yet implemented operations:

### Operations
- ^ (logical XOR);
- unary `~` (bit inversion);
- `>>`, `<<`, `>>>` (bit shifts);

### Predefined functions

- groupGenerator [#332](https://github.com/ergoplatform/sigma-rust/issues/332)
- xor
- substConstants
- downcast
- atLeast
- avlTree
- treeLookup
- xorOf

### Object properties and methods

#### GroupElement

- getEncoded [#330](https://github.com/ergoplatform/sigma-rust/issues/330)
- negate [#331](https://github.com/ergoplatform/sigma-rust/issues/331)

#### SigmaProp

- isProven

#### Box

- bytes
- bytesWithoutRef

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

- headers
- preHeader
- selfBoxIndex
- LastBlockUtxoRootHash

#### Collection

- indices [#314](https://github.com/ergoplatform/sigma-rust/issues/314)
- patch
- updated
- updateMany
- zip [#329](https://github.com/ergoplatform/sigma-rust/issues/329)

#### Option

- map
- filter



Implemented operations (IR nodes): 

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
- executeFromVar
- executeFromSelfReg
- getVar
- allZK
- anyZK
- decodePoint

### Object properties and methods

#### GroupElement

- exp [#297](https://github.com/ergoplatform/sigma-rust/issues/297)
- multiply [#298](https://github.com/ergoplatform/sigma-rust/issues/298)

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

#### Context

- dataInputs
- INPUTS
- OUTPUTS
- HEIGHT
- SELF
- minerPubKey
- getVar

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

#### Option

- isDefined
- get
- getOrElse

