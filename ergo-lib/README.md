[![Latest Version](https://img.shields.io/crates/v/ergo-lib.svg)](https://crates.io/crates/ergo-lib)
[![Documentation](https://docs.rs/ergo-lib/badge.svg)](https://docs.rs/crate/ergo-lib)

## Features:
- ErgoTree AST;
- Transactions, boxes, etc.;
- JSON serialization;
- Box builder(with mint token support);
- Transaction creation(builder) and signing;
- Box selection for funds and assets (with token burning support);


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

## Usage 
## Crate features
## `json` (default feature)
JSON serialization for chain types using `serde`.






