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
pub mod collections;
pub mod context_extension;
pub mod ergo_box;
pub mod ergo_state_ctx;
pub mod header;
pub mod secret_key;
pub mod transaction;
mod util;
pub use crate::error::*;
mod error;
