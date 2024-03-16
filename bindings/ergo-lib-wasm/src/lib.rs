//! WASM bindings for ergo-lib

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]
#![allow(unused_variables)]
// Clippy warnings
#![allow(clippy::new_without_default)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::unused_unit)]
#![deny(clippy::wildcard_enum_match_arm)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

pub mod address;
pub mod ast;
pub mod batchmerkleproof;
pub mod block_header;
pub mod box_coll;
pub mod box_selector;
pub mod context_extension;
pub mod contract;
pub mod data_input;
pub mod ergo_box;
pub mod ergo_state_ctx;
pub mod ergo_tree;
pub mod header;
pub mod input;
pub mod merkleproof;
pub mod nipopow;
pub mod parameters;

pub mod prover_result;
pub mod secret_key;
pub mod token;
pub mod transaction;
pub mod tx_builder;
pub mod utils;
pub mod verifier;
pub mod wallet;

mod error_conversion;
pub(crate) mod json;

#[cfg(feature = "rest")]
pub mod rest;
