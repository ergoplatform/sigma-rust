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
#![deny(rustdoc::broken_intra_doc_links)]

pub(crate) mod ast;
pub(crate) mod binder;
pub(crate) mod error;
pub(crate) mod hir;
pub(crate) mod lexer;
pub(crate) mod mir;
pub(crate) mod parser;
pub(crate) mod syntax;
pub(crate) mod type_infer;

pub mod compiler;
pub mod script_env;
