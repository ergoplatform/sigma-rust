//! ErgoScript compiler pipeline

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]
// Clippy exclusions
#![allow(clippy::unit_arg)]
#![deny(broken_intra_doc_links)]

mod ast;
mod binder;
mod hir;
mod lexer;
mod parser;
mod syntax;

mod compiler;
mod script_env;

pub use compiler::compile_hir;
pub use script_env::ScriptEnv;
