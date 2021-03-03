// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
// TODO: add docs and enable
// #![deny(missing_docs)]
// Clippy exclusions
#![allow(clippy::unit_arg)]
#![deny(broken_intra_doc_links)]

pub mod address;
pub mod ergo_tree;
pub mod ir_ergo_box;
pub mod mir;
pub mod serialization;
pub mod sigma_protocol;
pub mod type_check;
pub mod types;
pub mod util;
