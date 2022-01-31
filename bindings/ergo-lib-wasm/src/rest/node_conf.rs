//! Wasm API for ergo_rest::NodeConf
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// Node configuration
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, Copy, From, Into)]
pub struct NodeConf(ergo_lib::ergo_rest::NodeConf);
