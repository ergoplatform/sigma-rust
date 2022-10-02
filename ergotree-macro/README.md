# ergotree-macro

This crate defines a proc-macro `ergo_tree` that converts a pretty-printed representation of an ergo tree expression into an instance of `ergotree::ir::mir::expr::Expr`. This macro exists for the purpose of checking the correctness of the JIT v5 costing method.