# Architecture

This document describes the high-level architecture of ErgoScript compiler and ErgoTree interpreter.

## ErgoScript compiler 
ErgoScript compiler pass consists of the following phases:

### Lexer
Converts a source code into a list of tokens. Uses [`Logos`](https://docs.rs/logos/0.12.0/logos/index.html) under the hood.
Crate: `ergoscript-compiler`
Module: `lexer`

### Parser
Produces an AST by going through list of tokens. Uses [`Rowan`](https://docs.rs/rowan/0.12.6/rowan/index.html) CST (Concrete Syntax Trees) under the hood. AST nodes wrap Rowan's trees (CST) and expose node-specific details via methods (e.g. `BinaryExpr::lhs()`).
Crate: `ergoscript-compiler`
Modules: `parser, ast`

### HIR (High-level IR)
Created by "lowering" from AST produced by the parser. Each node(`hir::Expr`) has a kind(enum), span(source code reference) and an optional type.
Crate: `ergoscript-compiler`
Module: `hir`

### Binder
Rewrites HIR tree swapping identifiers (e.g. `HEIGHT`), some predefined functions (e.g. `min/max`) and variables from environment (`ScriptEnv`) with their HIR nodes.
Crate: `ergoscript-compiler`
Module: `binder`

### Type inference
Traverses the HIR tree and assigns a type to every node.
Crate: `ergoscript-compiler`
Module: `type_infer`

### MIR (Middle IR)
Created by "lowering" from HIR. "Final" IR, used in the interpreter and serialization.
Crate: `ergotree-ir`
Module: `mir`

### Type checking
Traverses the MIR tree and check that node's ancestors types correspond to node's type.
Crate: `ergotree-ir`
Module: `type_check`

All phases are run in `compiler::compile()` which is *the* entry point for the compiler.
Tests are comparing produced tree to the expected data(snapshot testing) using `expect_test` crate. 
To update test data, run `cargo test` with `UPDATE_EXPECT` variable:
```bash
env UPDATE_EXPECT=1 cargo test
```


## ErgoTree interpreter 
Evaluates MIR nodes by calling `Evaluable::eval()` on the tree root. Each node implements trait `Evaluable::eval()` method. 
Crate: `ergotree-interpreter`

## ErgoTree serialization
Each MIR node implements `SigmaSerializable` trait with `sigma_parse()` and `sigma_serialize()`.
Crate: `ergotree-ir`
Module: `serialization`



