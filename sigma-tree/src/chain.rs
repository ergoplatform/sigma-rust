//! Ergo chain types

#[cfg(feature = "with-serde")]
mod json;

mod base16_bytes;
// TODO: move to ergo_box
mod box_id;
mod digest32;

pub use base16_bytes::Base16DecodedBytes;
pub use base16_bytes::Base16EncodedBytes;
pub use box_id::*;
pub use digest32::*;

pub mod address;
pub mod context_extension;
pub mod contract;
pub mod data_input;
pub mod ergo_box;
pub mod ergo_state_context;
pub mod input;
pub mod prover_result;
pub mod token;
pub mod transaction;
