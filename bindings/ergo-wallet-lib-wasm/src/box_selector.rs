use sigma_tree::chain::ergo_box::ErgoBoxAssets;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum BoxSelector {
    SelectAll = 0,
}

impl BoxSelector {
    pub fn inner<T: ErgoBoxAssets>(
        &self,
    ) -> Box<dyn sigma_tree::wallet::box_selector::BoxSelector<T>> {
        match self {
            BoxSelector::SelectAll => {
                Box::new(sigma_tree::wallet::box_selector::simple::SimpleBoxSelector {})
            }
        }
    }
}
