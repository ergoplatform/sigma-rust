//! ErgoTree interpreter

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
#![deny(broken_intra_doc_links)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod contracts;
mod util;

pub mod eval;
// TODO: remove after https://github.com/ergoplatform/sigma-rust/pull/226 is merged
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
pub mod sigma_protocol;
