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

## Interpreter (what's implemented):
- global vars (`INPUTS`, `OUTPUTS`, `SELF`, `HEIGHT`);
- `Context` properties (`dataInputs`);
- Value types with serialization for most of them(types and values);
- `EQ`(`==`), `NEQ`(`!=`);
- `BlockValue`, `ValDef`, `ValUse`, `FuncValue`, `Apply`;
- `Fold`(collection), `ExtractAmount`(`Box.value`), `SelectField`(tuple field access); 
- `CalcBlake2b256`;
- Arithmetic ops (`+, -, *, /`);
- Comparison ops (`>, >=, <, <=`);
- `AND`, `OR`, `Collection` (collection declaration); 
- `BinAnd`, `BinOr`;
- `LogicalNot` (`!`);
- `Map`, `Filter` collection ops;
- `BoolToSigmaProp`;
- `If`;
- `Min`, `Max`;
- `ByIndex`, `Box.tokens`;
- `ExtractScriptBytes` (`Box.propositionBytes`);
- `SizeOf` (`Coll.size`);

### General

- :heavy_check_mark: Blocks (`BlockValue`);
- Variable definition (`ValDef`, `ValUse`);
- Function definition (`FuncValue`);
- Function application(`Apply`);

### Operations

- comparison: `>, <, >=, <=, ==, !=`;
- arithmetic: ` +, -, *, /, %`;
- logical: ` &&, ||`;
- | (bitwise OR);
- & (bitwise AND);
- ^ (logical XOR);
- ^ (bitwise XOR);
- `|` (byte-wise XOR of two collections of bytes);
- unary `!`;
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
- upcast
- allOf
- anyOf
- atLeast
- min
- max
- avlTree
- treeLookup
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

- propBytes
- isProven

#### Box

- value
- propositionBytes
- bytes
- bytesWithoutRef
- id
- creationInfo
- getReg
- tokens
- R0 .. R9

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

- dataInputs
- headers
- preHeader
- INPUTS
- OUTPUTS
- HEIGHT
- SELF
- selfBoxIndex
- LastBlockUtxoRootHash
- minerPubKey
- getVar

#### Collection

- size
- getOrElse
- map
- exists
- fold
- forall
- slice
- filter
- append
- apply
- indices
- flatMap
- patch
- updated
- updateMany
- indexOf
- zip

#### Option

- isDefined
- get
- getOrElse
- map
- filter


## Crate features
### `json` (default feature)
JSON serialization for chain types using `serde`.

