//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use ergo_lib::{
    chain::transaction::reduced::reduce_tx,
    ergotree_ir::{serialization::SigmaSerializable, sigma_protocol::sigma_boolean::SigmaBoolean},
};

use crate::{
    collections::ConstCollectionPtr,
    ergo_box::ErgoBox,
    ergo_state_ctx::ConstErgoStateContextPtr,
    transaction::{ConstUnsignedTransactionPtr, UnsignedTransaction, UnsignedTransactionPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Propositions list(public keys)
pub struct Propositions(pub(crate) Vec<SigmaBoolean>);
pub type PropositionsPtr = *mut Propositions;
pub type ConstPropositionsPtr = *const Propositions;

/// Create empty proposition holder
pub unsafe fn propositions_new(propositions_out: *mut PropositionsPtr) -> Result<(), Error> {
    let propositions_out = mut_ptr_as_mut(propositions_out, "propositions_out")?;
    *propositions_out = Box::into_raw(Box::new(Propositions(vec![])));
    Ok(())
}

/// Adding new proposition
pub unsafe fn propositions_add_proposition_from_bytes(
    propositions_mut: PropositionsPtr,
    bytes_ptr: *const u8,
    len: usize,
) -> Result<(), Error> {
    if bytes_ptr.is_null() {
        return Err(Error::Misc("bytes_ptr is null".into()));
    }
    let bytes = std::slice::from_raw_parts(bytes_ptr, len);
    let propositions_mut = mut_ptr_as_mut(propositions_mut, "propositions_mut")?;
    propositions_mut
        .0
        .push(SigmaBoolean::sigma_parse_bytes(bytes)?);
    Ok(())
}

/// Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
/// is augmented with ReducedInput which contains a script reduction result.
/// After an unsigned transaction is reduced it can be signed without context.
/// Thus, it can be serialized and transferred for example to Cold Wallet and signed
/// in an environment where secrets are known.
/// see EIP-19 for more details -
/// <https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md>
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ReducedTransaction(pub(crate) ergo_lib::chain::transaction::reduced::ReducedTransaction);
pub type ReducedTransactionPtr = *mut ReducedTransaction;
pub type ConstReducedTransactionPtr = *const ReducedTransaction;

/// Returns `reduced` transaction, i.e. unsigned transaction where each unsigned input
/// is augmented with ReducedInput which contains a script reduction result.
pub unsafe fn reduced_tx_from_unsigned_tx(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    state_context_ptr: ConstErgoStateContextPtr,
    reduced_tx_out: *mut ReducedTransactionPtr,
) -> Result<(), Error> {
    let data_boxes = const_ptr_as_ref(data_boxes_ptr, "data_boxes_ptr")?;
    let state_context = const_ptr_as_ref(state_context_ptr, "state_context_ptr")?;
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let boxes_to_spend = const_ptr_as_ref(boxes_to_spend_ptr, "boxes_to_spend_ptr")?;
    let reduced_tx_out = mut_ptr_as_mut(reduced_tx_out, "reduced_tx_out")?;
    let boxes_to_spend = boxes_to_spend.0.clone().into_iter().map(|b| b.0).collect();
    let data_boxes = data_boxes.0.clone().into_iter().map(|b| b.0).collect();
    let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
        unsigned_tx.0.clone(),
        boxes_to_spend,
        data_boxes,
    )?;
    let reduced_tx = reduce_tx(tx_context, &state_context.0).map(ReducedTransaction)?;
    *reduced_tx_out = Box::into_raw(Box::new(reduced_tx));
    Ok(())
}

/// Returns the unsigned transation
pub unsafe fn reduced_tx_unsigned_tx(
    reduced_tx_ptr: ConstReducedTransactionPtr,
    unsigned_tx_out: *mut UnsignedTransactionPtr,
) -> Result<(), Error> {
    let reduced_tx = const_ptr_as_ref(reduced_tx_ptr, "reduced_tx_ptr")?;
    let unsigned_tx_out = mut_ptr_as_mut(unsigned_tx_out, "unsigned_tx_out")?;
    *unsigned_tx_out = Box::into_raw(Box::new(UnsignedTransaction(
        reduced_tx.0.unsigned_tx.clone(),
    )));
    Ok(())
}
