//! WASM bindings for sigma-tree

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

pub mod address;
pub mod box_coll;
pub mod box_selector;
pub mod contract;
pub mod ergo_box;
pub mod ergo_state_ctx;
pub mod secret_key;
pub mod transaction;
pub mod tx_builder;
pub mod utils;
pub mod wallet;
