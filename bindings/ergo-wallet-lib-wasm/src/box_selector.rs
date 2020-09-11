use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum BoxSelector {
    SelectAll,
}

// impl BoxSelector {
//     pub fn inner<F, T: ErgoBoxAssets>(&self) -> F
//     where
//         F: Fn(Vec<T>, BoxValue, &[TokenAmount]) -> Result<BoxSelection<T>, BoxSelectorError>,
//     {
//         match self {
//             BoxSelector::SelectAll => {
//                 wallet::box_selector::select_all::select_all_box_selector::<T>
//             }
//         }
//     }
// }
