//! Simple box selection algorithms
use ergo_lib::chain;
use ergo_lib::wallet;
use ergo_lib::wallet::box_selector::BoxSelector;
use wasm_bindgen::prelude::*;

use crate::box_coll::ErgoBoxes;
use crate::ergo_box::BoxValue;
use crate::token::Tokens;

/// Selected boxes with change boxes (by [`BoxSelector`])
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxSelection(wallet::box_selector::BoxSelection<ergo_lib::chain::ergo_box::ErgoBox>);

impl From<BoxSelection> for wallet::box_selector::BoxSelection<chain::ergo_box::ErgoBox> {
    fn from(v: BoxSelection) -> Self {
        v.0
    }
}

/// Naive box selector, collects inputs until target balance is reached
#[wasm_bindgen]
pub struct SimpleBoxSelector(wallet::box_selector::SimpleBoxSelector);

#[wasm_bindgen]
impl SimpleBoxSelector {
    /// Create empty DataInputs
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
        let target_tokens: Vec<chain::token::Token> = target_tokens.clone().into();
        self.0
            .select(
                inputs.clone().into(),
                target_balance.clone().into(),
                target_tokens.as_slice(),
            )
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))
            .map(BoxSelection)
    }
}
