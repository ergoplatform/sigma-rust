//! Ergo transaction

use std::convert::{TryFrom, TryInto};

use ergo_lib::{
    chain,
    ergo_chain_types::{Base16DecodedBytes, Base16EncodedBytes},
};

use crate::{
    collections::{Collection, CollectionPtr, ConstCollectionPtr},
    data_input::DataInput,
    ergo_box::{ErgoBox, ErgoBoxCandidate},
    ergo_state_ctx::ConstErgoStateContextPtr,
    input::{Input, UnsignedInput},
    json::{TransactionJsonEip12, UnsignedTransactionJsonEip12},
    reduced::ConstPropositionsPtr,
    util::{const_ptr_as_ref, mut_ptr_as_mut, ByteArray},
    Error,
};

/// CommitmentHint
pub struct CommitmentHint(
    pub(crate) ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::CommitmentHint,
);
pub type CommitmentHintPtr = *mut CommitmentHint;
pub type ConstCommitmentHintPtr = *const CommitmentHint;

/// `HintsBag`
pub struct HintsBag(
    pub(crate) ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::HintsBag,
);
pub type HintsBagPtr = *mut HintsBag;
pub type ConstHintsBagPtr = *const HintsBag;

/// Empty HintsBag
pub unsafe fn hints_bag_empty(hints_bag_out: *mut HintsBagPtr) -> Result<(), Error> {
    let hints_bag_out = mut_ptr_as_mut(hints_bag_out, "hints_bag_out")?;
    *hints_bag_out = Box::into_raw(Box::new(HintsBag(
        ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::HintsBag::empty(),
    )));
    Ok(())
}

/// Add commitment hint to the bag
pub unsafe fn hints_bag_add_commitment(
    hints_bag_mut: HintsBagPtr,
    hint_ptr: ConstCommitmentHintPtr,
) -> Result<(), Error> {
    let hints_bag_mut = mut_ptr_as_mut(hints_bag_mut, "hints_bag_mut")?;
    let hint = const_ptr_as_ref(hint_ptr, "hint_ptr")?;
    hints_bag_mut.0.add_hint(
        ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::Hint::CommitmentHint(
            hint.0.clone(),
        ),
    );
    Ok(())
}

/// Length of HintsBag
pub unsafe fn hints_bag_len(hints_bag_ptr: ConstHintsBagPtr) -> Result<usize, Error> {
    let hints_bag = const_ptr_as_ref(hints_bag_ptr, "hints_bag_ptr")?;
    Ok(hints_bag.0.hints.len())
}

/// Get commitment
pub unsafe fn hints_bag_get(
    hints_bag_ptr: ConstHintsBagPtr,
    index: usize,
    hint_out: *mut CommitmentHintPtr,
) -> Result<bool, Error> {
    let hints_bag = const_ptr_as_ref(hints_bag_ptr, "hints_bag_ptr")?;
    let hint_out = mut_ptr_as_mut(hint_out, "hint_out")?;
    if let Some(commitment) = hints_bag.0.commitments().get(index) {
        *hint_out = Box::into_raw(Box::new(CommitmentHint(commitment.clone())));
        return Ok(true);
    }
    Ok(false)
}

/// TransactionHintsBag
pub struct TransactionHintsBag(pub(crate) ergo_lib::wallet::multi_sig::TransactionHintsBag);
pub type TransactionHintsBagPtr = *mut TransactionHintsBag;
pub type ConstTransactionHintsBagPtr = *const TransactionHintsBag;

/// Empty TransactionHintsBag
pub unsafe fn transaction_hints_bag_empty(
    transaction_hints_bag_out: *mut TransactionHintsBagPtr,
) -> Result<(), Error> {
    let transaction_hints_bag_out =
        mut_ptr_as_mut(transaction_hints_bag_out, "transaction_hints_bag_out")?;
    *transaction_hints_bag_out = Box::into_raw(Box::new(TransactionHintsBag(
        ergo_lib::wallet::multi_sig::TransactionHintsBag::empty(),
    )));
    Ok(())
}

/// Adding hints for input
pub unsafe fn transaction_hints_bag_add_hints_for_input(
    transaction_hints_bag_mut: TransactionHintsBagPtr,
    index: usize,
    hints_bag_ptr: ConstHintsBagPtr,
) -> Result<(), Error> {
    let transaction_hints_bag_mut =
        mut_ptr_as_mut(transaction_hints_bag_mut, "transaction_hints_bag_mut")?;
    let hints_bag = const_ptr_as_ref(hints_bag_ptr, "hints_bag_ptr")?;
    transaction_hints_bag_mut
        .0
        .add_hints_for_input(index, hints_bag.0.clone());
    Ok(())
}

/// Get HintsBag corresponding to input index
pub unsafe fn transaction_hints_bag_all_hints_for_input(
    transaction_hints_bag_ptr: ConstTransactionHintsBagPtr,
    index: usize,
    hints_bag_out: *mut HintsBagPtr,
) -> Result<(), Error> {
    let transaction_hints_bag =
        const_ptr_as_ref(transaction_hints_bag_ptr, "transaction_hints_bag_ptr")?;
    let hints_bag_out = mut_ptr_as_mut(hints_bag_out, "hints_bag_out")?;
    *hints_bag_out = Box::into_raw(Box::new(HintsBag(
        transaction_hints_bag.0.all_hints_for_input(index),
    )));
    Ok(())
}

/// Extract hints from signed transaction
pub unsafe fn transaction_extract_hints(
    signed_transaction_ptr: ConstTransactionPtr,
    state_context_ptr: ConstErgoStateContextPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    real_propositions_ptr: ConstPropositionsPtr,
    simulated_propositions_ptr: ConstPropositionsPtr,
    transaction_hints_bag_out: *mut TransactionHintsBagPtr,
) -> Result<(), Error> {
    let signed_transaction = const_ptr_as_ref(signed_transaction_ptr, "signed_transaction_ptr")?;
    let state_context = const_ptr_as_ref(state_context_ptr, "state_context_ptr")?;
    let boxes_to_spend = const_ptr_as_ref(boxes_to_spend_ptr, "boxes_to_spend_ptr")?;
    let data_boxes = const_ptr_as_ref(data_boxes_ptr, "data_boxes_ptr")?;
    let real_propositions = const_ptr_as_ref(real_propositions_ptr, "real_propositions_ptr")?;
    let simulated_propositions =
        const_ptr_as_ref(simulated_propositions_ptr, "simulated_propositions_ptr")?;
    let transaction_hints_bag_out =
        mut_ptr_as_mut(transaction_hints_bag_out, "transaction_hints_bag_out")?;

    let boxes_to_spend = boxes_to_spend.0.clone().into_iter().map(|x| x.0).collect();
    let data_boxes = data_boxes.0.clone().into_iter().map(|x| x.0).collect();
    let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
        signed_transaction.0.clone(),
        boxes_to_spend,
        data_boxes,
    )?;

    *transaction_hints_bag_out = Box::into_raw(Box::new(TransactionHintsBag(
        ergo_lib::wallet::multi_sig::extract_hints(
            &tx_context,
            &state_context.0.clone(),
            real_propositions.0.clone(),
            simulated_propositions.0.clone(),
        )?,
    )));
    Ok(())
}

/// Unsigned (inputs without proofs) transaction
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UnsignedTransaction(pub(crate) chain::transaction::unsigned::UnsignedTransaction);
pub type UnsignedTransactionPtr = *mut UnsignedTransaction;
pub type ConstUnsignedTransactionPtr = *const UnsignedTransaction;

/// Get id for transaction
pub unsafe fn unsigned_tx_id(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    tx_id_out: *mut TxIdPtr,
) -> Result<(), Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let tx_id_out = mut_ptr_as_mut(tx_id_out, "tx_id_out")?;
    *tx_id_out = Box::into_raw(Box::new(TxId(unsigned_tx.0.id())));
    Ok(())
}

/// Inputs for transaction
pub unsafe fn unsigned_tx_inputs(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    unsigned_inputs_out: *mut CollectionPtr<UnsignedInput>,
) -> Result<(), Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let unsigned_inputs_out = mut_ptr_as_mut(unsigned_inputs_out, "unsigned_inputs_out")?;
    *unsigned_inputs_out = Box::into_raw(Box::new(Collection(
        unsigned_tx
            .0
            .inputs
            .as_vec()
            .clone()
            .into_iter()
            .map(UnsignedInput)
            .collect(),
    )));
    Ok(())
}

/// Data inputs for transaction
pub unsafe fn unsigned_tx_data_inputs(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    data_inputs_out: *mut CollectionPtr<DataInput>,
) -> Result<(), Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let data_inputs_out = mut_ptr_as_mut(data_inputs_out, "data_inputs_out")?;
    *data_inputs_out = Box::into_raw(Box::new(Collection(
        unsigned_tx
            .0
            .data_inputs
            .as_ref()
            .map(|v| v.as_vec().clone())
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(DataInput)
            .collect(),
    )));
    Ok(())
}

/// Output candidates for transaction
pub unsafe fn unsigned_tx_output_candidates(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    ergo_box_candidates_out: *mut CollectionPtr<ErgoBoxCandidate>,
) -> Result<(), Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let ergo_box_candidates_out =
        mut_ptr_as_mut(ergo_box_candidates_out, "ergo_box_candidates_out")?;
    *ergo_box_candidates_out = Box::into_raw(Box::new(Collection(
        unsigned_tx
            .0
            .output_candidates
            .iter()
            .map(|ebc| ErgoBoxCandidate(ebc.clone()))
            .collect(),
    )));
    Ok(())
}

/// Parse from JSON. Supports Ergo Node/Explorer API and box values and token amount encoded as
/// strings
pub unsafe fn unsigned_tx_from_json(
    json: &str,
    unsigned_tx_out: *mut UnsignedTransactionPtr,
) -> Result<(), Error> {
    let unsigned_tx_out = mut_ptr_as_mut(unsigned_tx_out, "unsigned_tx_out")?;
    let unsigned_tx = serde_json::from_str(json).map(UnsignedTransaction)?;
    *unsigned_tx_out = Box::into_raw(Box::new(unsigned_tx));
    Ok(())
}

/// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
pub unsafe fn unsigned_tx_to_json(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
) -> Result<String, Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let s = serde_json::to_string(&unsigned_tx.0)?;
    Ok(s)
}

/// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
pub unsafe fn unsigned_tx_to_json_eip12(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
) -> Result<String, Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let tx_dapp: UnsignedTransactionJsonEip12 = unsigned_tx.0.clone().into();
    let s = serde_json::to_string(&tx_dapp)?;
    Ok(s)
}

/// Transaction id
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TxId(pub(crate) chain::transaction::TxId);
pub type TxIdPtr = *mut TxId;
pub type ConstTxIdPtr = *const TxId;

/// Convert a hex string into a TxId
pub unsafe fn tx_id_from_str(str: &str, tx_id_out: *mut TxIdPtr) -> Result<(), Error> {
    let tx_id_out = mut_ptr_as_mut(tx_id_out, "tx_id_out")?;
    let bytes = Base16DecodedBytes::try_from(str.to_string())?;
    let tx_id = bytes
        .try_into()
        .map(|digest| TxId(chain::transaction::TxId(digest)))?;
    *tx_id_out = Box::into_raw(Box::new(tx_id));
    Ok(())
}

/// Get the tx id as bytes
pub unsafe fn tx_id_to_str(tx_id_ptr: ConstTxIdPtr) -> Result<String, Error> {
    let tx_id = const_ptr_as_ref(tx_id_ptr, "tx_id_ptr")?;
    let base16_bytes = Base16EncodedBytes::new(tx_id.0 .0 .0.as_ref());
    Ok(base16_bytes.into())
}

/**
 * ErgoTransaction is an atomic state transition operation. It destroys Boxes from the state
 * and creates new ones. If transaction is spending boxes protected by some non-trivial scripts,
 * its inputs should also contain proof of spending correctness - context extension (user-defined
 * key-value map) and data inputs (links to existing boxes in the state) that may be used during
 * script reduction to crypto, signatures that satisfies the remaining cryptographic protection
 * of the script.
 * Transactions are not encrypted, so it is possible to browse and view every transaction ever
 * collected into a block.
 */
pub struct Transaction(pub(crate) chain::transaction::Transaction);
pub type TransactionPtr = *mut Transaction;
pub type ConstTransactionPtr = *const Transaction;

/// Create Transaction from UnsignedTransaction and an array of proofs in the same order as
/// UnsignedTransaction.inputs with empty proof indicated with empty byte array
pub unsafe fn tx_from_unsigned_tx(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    proofs_ptr: ConstCollectionPtr<ByteArray>,
    tx_out: *mut TransactionPtr,
) -> Result<(), Error> {
    let proofs = const_ptr_as_ref(proofs_ptr, "proofs_ptr")?;
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let tx_out = mut_ptr_as_mut(tx_out, "tx_out")?;
    let tx = chain::transaction::Transaction::from_unsigned_tx(
        unsigned_tx.0.clone(),
        proofs
            .0
            .iter()
            .cloned()
            .map(|bytes| bytes.0.into())
            .collect(),
    )
    .map(Transaction)?;
    *tx_out = Box::into_raw(Box::new(tx));
    Ok(())
}

/// Get id for transaction
pub unsafe fn tx_id(tx_ptr: ConstTransactionPtr, tx_id_out: *mut TxIdPtr) -> Result<(), Error> {
    let tx = const_ptr_as_ref(tx_ptr, "tx_ptr")?;
    let tx_id_out = mut_ptr_as_mut(tx_id_out, "tx_id_out")?;
    *tx_id_out = Box::into_raw(Box::new(TxId(tx.0.id())));
    Ok(())
}

/// Parse from JSON. Supports Ergo Node/Explorer API and box values and token amount encoded as
/// strings
pub unsafe fn tx_from_json(json: &str, tx_out: *mut TransactionPtr) -> Result<(), Error> {
    let tx_out = mut_ptr_as_mut(tx_out, "tx_out")?;
    let tx = serde_json::from_str(json).map(Transaction)?;
    *tx_out = Box::into_raw(Box::new(tx));
    Ok(())
}

/// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
pub unsafe fn tx_to_json(tx_ptr: ConstTransactionPtr) -> Result<String, Error> {
    let tx = const_ptr_as_ref(tx_ptr, "tx_ptr")?;
    let s = serde_json::to_string(&tx.0)?;
    Ok(s)
}

/// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
pub unsafe fn tx_to_json_eip12(tx_ptr: ConstTransactionPtr) -> Result<String, Error> {
    let tx = const_ptr_as_ref(tx_ptr, "tx_ptr")?;
    let tx_dapp: TransactionJsonEip12 = tx.0.clone().into();
    let s = serde_json::to_string(&tx_dapp)?;
    Ok(s)
}

/// Inputs for transaction
pub unsafe fn tx_inputs(
    tx_ptr: ConstTransactionPtr,
    inputs_out: *mut CollectionPtr<Input>,
) -> Result<(), Error> {
    let tx = const_ptr_as_ref(tx_ptr, "tx_ptr")?;
    let inputs_out = mut_ptr_as_mut(inputs_out, "inputs_out")?;
    *inputs_out = Box::into_raw(Box::new(Collection(
        tx.0.inputs
            .as_vec()
            .clone()
            .into_iter()
            .map(Input)
            .collect(),
    )));
    Ok(())
}

/// Data inputs for transaction
pub unsafe fn tx_data_inputs(
    tx_ptr: ConstTransactionPtr,
    data_inputs_out: *mut CollectionPtr<DataInput>,
) -> Result<(), Error> {
    let tx = const_ptr_as_ref(tx_ptr, "tx_ptr")?;
    let data_inputs_out = mut_ptr_as_mut(data_inputs_out, "data_inputs_out")?;
    *data_inputs_out = Box::into_raw(Box::new(Collection(
        tx.0.data_inputs
            .as_ref()
            .map(|v| v.as_vec().clone())
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(DataInput)
            .collect(),
    )));
    Ok(())
}

/// Output candidates for transaction
pub unsafe fn tx_output_candidates(
    tx_ptr: ConstTransactionPtr,
    ergo_box_candidates_out: *mut CollectionPtr<ErgoBoxCandidate>,
) -> Result<(), Error> {
    let tx = const_ptr_as_ref(tx_ptr, "tx_ptr")?;
    let ergo_box_candidates_out =
        mut_ptr_as_mut(ergo_box_candidates_out, "ergo_box_candidates_out")?;
    *ergo_box_candidates_out = Box::into_raw(Box::new(Collection(
        tx.0.output_candidates
            .iter()
            .map(|ebc| ErgoBoxCandidate(ebc.clone()))
            .collect(),
    )));
    Ok(())
}

/// Returns ErgoBox's created from ErgoBoxCandidate's with tx id and indices
pub unsafe fn tx_outputs(
    tx_ptr: ConstTransactionPtr,
    ergo_box_out: *mut CollectionPtr<ErgoBox>,
) -> Result<(), Error> {
    let tx = const_ptr_as_ref(tx_ptr, "tx_ptr")?;
    let ergo_box_out = mut_ptr_as_mut(ergo_box_out, "ergo_box_candidates_out")?;
    *ergo_box_out = Box::into_raw(Box::new(Collection(
        tx.0.outputs
            .iter()
            .map(|ebc| ErgoBox(ebc.clone()))
            .collect(),
    )));
    Ok(())
}

/// Validate a transaction
pub unsafe fn tx_validate(
    tx_ptr: ConstTransactionPtr,
    state_context_ptr: ConstErgoStateContextPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
) -> Result<(), Error> {
    let state_context = const_ptr_as_ref(state_context_ptr, "state_context_ptr")?;
    let tx = const_ptr_as_ref(tx_ptr, "tx_ptr")?;
    let boxes_to_spend = const_ptr_as_ref(boxes_to_spend_ptr, "boxes_to_spend_ptr")?;
    let data_boxes = const_ptr_as_ref(data_boxes_ptr, "data_boxes_ptr")?;
    let boxes_to_spend = boxes_to_spend.0.clone().into_iter().map(|b| b.0).collect();
    let data_boxes = data_boxes.0.clone().into_iter().map(|b| b.0).collect();
    let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
        tx.0.clone(),
        boxes_to_spend,
        data_boxes,
    )?;
    tx_context.validate(&state_context.0)?;
    Ok(())
}
