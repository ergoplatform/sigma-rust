//! C compatible functions to use in C and JNI bindings

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
// #![deny(missing_docs)]
#![allow(clippy::missing_safety_doc)]

pub mod address;
pub mod block_header;
pub mod box_builder;
pub mod box_selector;
pub mod collections;
pub mod constant;
pub mod context_extension;
pub mod contract;
pub mod data_input;
pub mod ergo_box;
pub mod ergo_state_ctx;
pub mod ergo_tree;
pub mod error_conversion;
pub mod ext_secret_key;
pub mod header;
pub mod input;
mod json;
pub mod merkleproof;
pub mod reduced;
pub mod secret_key;
pub mod token;
pub mod transaction;
pub mod tx_builder;
pub mod util;
pub mod wallet;
pub use crate::error::*;
mod error;
