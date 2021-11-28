//! Simple box selection algorithms
use ergo_lib::wallet::{self, box_selector::BoxSelector};

use crate::{
    collections::{Collection, CollectionPtr, ConstCollectionPtr},
    ergo_box::{ConstBoxValuePtr, ErgoBox, ErgoBoxAssetsData},
    token::ConstTokensPtr,
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Selected boxes with change boxes (by [`BoxSelector`])
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxSelection(
    wallet::box_selector::BoxSelection<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox>,
);
pub type BoxSelectionPtr = *mut BoxSelection;
pub type ConstBoxSelectionPtr = *const BoxSelection;

pub unsafe fn box_selection_new(
    ergo_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    change_ergo_boxes_ptr: ConstCollectionPtr<ErgoBoxAssetsData>,
    box_selection_out: *mut BoxSelectionPtr,
) -> Result<(), Error> {
    let ergo_boxes = const_ptr_as_ref(ergo_boxes_ptr, "ergo_boxes_ptr")?;
    let change_ergo_boxes = const_ptr_as_ref(change_ergo_boxes_ptr, "change_ergo_boxes_ptr")?;
    let box_selection_out = mut_ptr_as_mut(box_selection_out, "box_selection_out")?;
    let boxes = wallet::box_selector::SelectedBoxes::from_vec(
        ergo_boxes.0.clone().into_iter().map(|b| b.0).collect(),
    )
    .map_err(|_| Error::Misc("BoxSelection: can't form boxes".into()))?;
    *box_selection_out = Box::into_raw(Box::new(BoxSelection(
        wallet::box_selector::BoxSelection::<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox> {
            boxes,
            change_boxes: change_ergo_boxes
                .0
                .clone()
                .into_iter()
                .map(|b| b.0)
                .collect(),
        },
    )));
    Ok(())
}

pub unsafe fn box_selection_boxes(
    box_selection_ptr: ConstBoxSelectionPtr,
    ergo_boxes_out: *mut CollectionPtr<ErgoBox>,
) -> Result<(), Error> {
    let box_selection = const_ptr_as_ref(box_selection_ptr, "box_selection_ptr")?;
    let ergo_boxes_out = mut_ptr_as_mut(ergo_boxes_out, "ergo_boxes_out")?;
    *ergo_boxes_out = Box::into_raw(Box::new(Collection(
        box_selection
            .0
            .boxes
            .clone()
            .into_iter()
            .map(ErgoBox)
            .collect(),
    )));
    Ok(())
}

pub unsafe fn box_selection_change(
    box_selection_ptr: ConstBoxSelectionPtr,
    change_ergo_boxes_out: *mut CollectionPtr<ErgoBoxAssetsData>,
) -> Result<(), Error> {
    let box_selection = const_ptr_as_ref(box_selection_ptr, "box_selection_ptr")?;
    let change_ergo_boxes_out = mut_ptr_as_mut(change_ergo_boxes_out, "change_ergo_boxes_out")?;
    *change_ergo_boxes_out = Box::into_raw(Box::new(Collection(
        box_selection
            .0
            .change_boxes
            .clone()
            .into_iter()
            .map(ErgoBoxAssetsData)
            .collect(),
    )));
    Ok(())
}

/// Naive box selector, collects inputs until target balance is reached
pub struct SimpleBoxSelector(wallet::box_selector::SimpleBoxSelector);
pub type SimpleBoxSelectorPtr = *mut SimpleBoxSelector;
pub type ConstSimpleBoxSelectorPtr = *const SimpleBoxSelector;

pub unsafe fn simple_box_selector_new(
    simple_box_selector_out: *mut SimpleBoxSelectorPtr,
) -> Result<(), Error> {
    let simple_box_selector_out =
        mut_ptr_as_mut(simple_box_selector_out, "simple_box_selector_out")?;
    *simple_box_selector_out = Box::into_raw(Box::new(SimpleBoxSelector(
        wallet::box_selector::SimpleBoxSelector::new(),
    )));
    Ok(())
}

pub unsafe fn simple_box_selector_select(
    simple_box_selector_ptr: ConstSimpleBoxSelectorPtr,
    inputs_ptr: ConstCollectionPtr<ErgoBox>,
    target_balance_ptr: ConstBoxValuePtr,
    target_tokens_ptr: ConstTokensPtr,
    box_selection_out: *mut BoxSelectionPtr,
) -> Result<(), Error> {
    let inputs = const_ptr_as_ref(inputs_ptr, "inputs_ptr")?;
    let target_balance = const_ptr_as_ref(target_balance_ptr, "target_balance_ptr")?;
    let target_tokens = const_ptr_as_ref(target_tokens_ptr, "target_tokens_ptr")?;
    let simple_box_selector = const_ptr_as_ref(simple_box_selector_ptr, "simple_box_selector_ptr")?;
    let box_selection_out = mut_ptr_as_mut(box_selection_out, "box_selection_out")?;
    let box_selection = simple_box_selector
        .0
        .select(
            inputs.0.clone().into_iter().map(|b| b.0).collect(),
            target_balance.0,
            &target_tokens
                .0
                .clone()
                .map(|tokens| tokens.mapped(|t| t.0).as_vec().clone())
                .unwrap_or_else(Vec::new),
        )
        .map_err(|_| Error::Misc("SimpleBoxSelection::select(): error".into()))?;
    *box_selection_out = Box::into_raw(Box::new(BoxSelection(box_selection)));
    Ok(())
}
