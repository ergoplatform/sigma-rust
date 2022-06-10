//! Node info

use derive_more::{From, Into};
use wasm_bindgen::prelude::*;

/// Node info
#[wasm_bindgen]
#[derive(Debug, Clone, From, Into)]
pub struct NodeInfo(pub(crate) ergo_lib::ergo_rest::NodeInfo);

#[wasm_bindgen]
impl NodeInfo {
    /// Get name of the ergo node
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    /// Returns true iff the ergo node is at least v4.0.28. This is important since nipopow proofs
    /// only work correctly from this version onwards.
    pub fn is_at_least_version_4_0_28(&self) -> bool {
        self.0.is_at_least_version_4_0_28()
    }
}
