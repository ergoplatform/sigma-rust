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

mod big_integer;
mod types;

pub mod ast;
pub mod chain;
pub mod constants;
pub mod eval;
pub mod serialization;
pub mod sigma_protocol;
pub mod util;
pub mod wallet;

pub mod ergo_tree;

#[cfg(test)]
pub mod test_util;
