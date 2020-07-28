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
// Clippy exclusions
#![allow(clippy::unit_arg)]

pub mod ast;
mod big_integer;
mod constants;
mod ecpoint;
mod ergo_tree;
mod eval;
mod serialization;
mod types;

pub mod chain;
pub mod sigma_protocol;

pub use ergo_tree::*;
