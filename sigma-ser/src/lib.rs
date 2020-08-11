//! Sigma serialization

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// io::Read wrapper with `peek` operation
pub mod peekable_reader;
/// VLQ encoder
pub mod vlq_encode;
/// ZigZag encoder
pub mod zig_zag_encode;
