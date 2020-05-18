//! Ergo blockchain entities

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]

mod misc;
mod utils;

pub use misc::*;

pub mod box_id;
pub mod constants;
pub mod context_extension;
pub mod data_input;
pub mod ergo_box;
pub mod input;
pub mod prover_result;
pub mod token;
pub mod transaction;
