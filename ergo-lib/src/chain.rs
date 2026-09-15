//! Ergo chain types

#[cfg(feature = "json")]
pub mod json;

pub mod block;
pub mod contract;
pub mod emission;
pub mod ergo_box;
pub mod ergo_state_context;
pub mod ergo_tree_predef;
pub mod genesis;
pub mod parameters;
pub mod transaction;
