//! Box selection algorithms
use ergo_lib::chain::ergo_box::ErgoBoxAssets;
use wasm_bindgen::prelude::*;

/// Box selector implementations
#[wasm_bindgen]
pub enum BoxSelector {
    /// Naive box selector, collects inputs until target balance is reached
    Simple = 0,
}

impl BoxSelector {
    /// Get underlying ergo-lib BoxSelector implementation
    pub fn inner<T: ErgoBoxAssets>(
        &self,
    ) -> Box<dyn ergo_lib::wallet::box_selector::BoxSelector<T>> {
        match self {
            BoxSelector::Simple => {
                Box::new(ergo_lib::wallet::box_selector::simple::SimpleBoxSelector {})
            }
        }
    }
}
