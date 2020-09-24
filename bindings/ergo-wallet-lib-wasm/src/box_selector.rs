//! Box selection algorithms
use sigma_tree::chain::ergo_box::ErgoBoxAssets;
use wasm_bindgen::prelude::*;

/// Box selector implementations
#[wasm_bindgen]
pub enum BoxSelector {
    /// Naive box selector, collects inputs until target balance is reached
    Simple = 0,
}

impl BoxSelector {
    /// Get underlying sigma-tree BoxSelector implementation
    pub fn inner<T: ErgoBoxAssets>(
        &self,
    ) -> Box<dyn sigma_tree::wallet::box_selector::BoxSelector<T>> {
        match self {
            BoxSelector::Simple => {
                Box::new(sigma_tree::wallet::box_selector::simple::SimpleBoxSelector {})
            }
        }
    }
}
