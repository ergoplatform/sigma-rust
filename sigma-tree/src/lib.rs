//! ErgoTree IR

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]

mod ast;
mod constants;
mod ecpoint;
mod ergo_tree;
mod eval;
mod serialization;
mod sigma_protocol;
mod types;

pub mod chain;
pub use ergo_tree::*;
