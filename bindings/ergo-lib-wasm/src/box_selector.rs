//! Simple box selection algorithms
use ergo_lib::ergotree_ir::chain;
use ergo_lib::wallet;
use ergo_lib::wallet::box_selector::BoxSelector;
use wasm_bindgen::prelude::*;

use crate::box_coll::ErgoBoxes;
use crate::ergo_box::BoxValue;
use crate::ergo_box::ErgoBoxAssetsDataList;
use crate::error_conversion::to_js;
use crate::token::Tokens;

extern crate derive_more;
use derive_more::{From, Into};

/// Selected boxes with change boxes (by [`BoxSelector`])
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct BoxSelection(
    wallet::box_selector::BoxSelection<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox>,
);

#[wasm_bindgen]
impl BoxSelection {
    /// Create a selection to easily inject custom selection algorithms
    #[wasm_bindgen(constructor)]
    pub fn new(boxes: &ErgoBoxes, change: &ErgoBoxAssetsDataList) -> Self {
        BoxSelection(wallet::box_selector::BoxSelection::<
            ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox,
        > {
            boxes: boxes.clone().into(),
            change_boxes: change.clone().into(),
        })
    }

    /// Selected boxes to spend as transaction inputs
    pub fn boxes(&self) -> ErgoBoxes {
        self.0.boxes.clone().into()
    }

    /// Selected boxes to use as change
    pub fn change(&self) -> ErgoBoxAssetsDataList {
        self.0.change_boxes.clone().into()
    }
}

/// Naive box selector, collects inputs until target balance is reached
#[wasm_bindgen]
pub struct SimpleBoxSelector(wallet::box_selector::SimpleBoxSelector);

#[wasm_bindgen]
impl SimpleBoxSelector {
    /// Create empty SimpleBoxSelector
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        SimpleBoxSelector(wallet::box_selector::SimpleBoxSelector::new())
    }

    /// Selects inputs to satisfy target balance and tokens.
    /// `inputs` - available inputs (returns an error, if empty),
    /// `target_balance` - coins (in nanoERGs) needed,
    /// `target_tokens` - amount of tokens needed.
    /// Returns selected inputs and box assets(value+tokens) with change.
    pub fn select(
        &self,
        inputs: &ErgoBoxes,
        target_balance: &BoxValue,
        target_tokens: &Tokens,
    ) -> Result<BoxSelection, JsValue> {
        let target_tokens: Option<chain::ergo_box::BoxTokens> = target_tokens.clone().into();
        self.0
            .select(
                inputs.clone().into(),
                target_balance.clone().into(),
                target_tokens
                    .as_ref()
                    .map(chain::ergo_box::BoxTokens::as_slice)
                    .unwrap_or(&[]),
            )
            .map_err(to_js)
            .map(BoxSelection)
    }
}
