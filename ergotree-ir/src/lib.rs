//! ErgoTree, MIR (Middle-level Internal Representation)

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
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
//#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

mod has_opcode;

pub mod base16_str;
pub mod bigint256;
pub mod chain;
pub mod ergo_tree;
pub mod mir;
pub mod serialization;
pub mod sigma_protocol;
pub mod type_check;
pub mod types;
pub mod util;

#[cfg(feature = "ergotree-proc-macro")]
pub mod ergotree_proc_macro;
