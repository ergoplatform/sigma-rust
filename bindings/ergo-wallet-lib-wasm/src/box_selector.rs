use sigma_tree::chain::ergo_box::ErgoBox;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum BoxSelector {
    SelectAll,
}

impl BoxSelector {
    pub fn inner(&self) -> Box<dyn sigma_tree::wallet::box_selector::BoxSelector<Item = ErgoBox>> {
        match self {
            BoxSelector::SelectAll => {
                Box::new(sigma_tree::wallet::box_selector::select_all::SelectAllBoxSelector::new())
            }
        }
    }
}
