use ergo_lib_c_core::{
    box_selector::*,
    collections::{CollectionPtr, ConstCollectionPtr},
    ergo_box::{ConstBoxValuePtr, ErgoBox, ErgoBoxAssetsData},
    token::ConstTokensPtr,
    Error,
};

use paste::paste;

use crate::{delete_ptr, ErrorPtr};

// `BoxSelection` bindings -------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_selection_new(
    ergo_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    change_ergo_boxes_ptr: ConstCollectionPtr<ErgoBoxAssetsData>,
    box_selection_out: *mut BoxSelectionPtr,
) {
    #[allow(clippy::unwrap_used)]
    box_selection_new(ergo_boxes_ptr, change_ergo_boxes_ptr, box_selection_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_selection_boxes(
    box_selection_ptr: ConstBoxSelectionPtr,
    ergo_boxes_out: *mut CollectionPtr<ErgoBox>,
) {
    #[allow(clippy::unwrap_used)]
    box_selection_boxes(box_selection_ptr, ergo_boxes_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_box_selection_change(
    box_selection_ptr: ConstBoxSelectionPtr,
    change_ergo_boxes_out: *mut CollectionPtr<ErgoBoxAssetsData>,
) {
    #[allow(clippy::unwrap_used)]
    box_selection_change(box_selection_ptr, change_ergo_boxes_out).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_box_selection_delete(ptr: BoxSelectionPtr) {
    unsafe { delete_ptr(ptr) }
}

make_ffi_eq!(BoxSelection);

// `SimpleBoxSelector` bindings ---------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_simple_box_selector_new(
    simple_box_selector_out: *mut SimpleBoxSelectorPtr,
) {
    #[allow(clippy::unwrap_used)]
    simple_box_selector_new(simple_box_selector_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_simple_box_selector_select(
    simple_box_selector_ptr: ConstSimpleBoxSelectorPtr,
    inputs_ptr: ConstCollectionPtr<ErgoBox>,
    target_balance_ptr: ConstBoxValuePtr,
    target_tokens_ptr: ConstTokensPtr,
    box_selection_out: *mut BoxSelectionPtr,
) -> ErrorPtr {
    let res = simple_box_selector_select(
        simple_box_selector_ptr,
        inputs_ptr,
        target_balance_ptr,
        target_tokens_ptr,
        box_selection_out,
    );
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_simple_box_selector_delete(ptr: SimpleBoxSelectorPtr) {
    unsafe { delete_ptr(ptr) }
}
