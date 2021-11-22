//! Ergo transaction

use ergo_lib::{chain, ergotree_ir::chain::base16_bytes::Base16EncodedBytes};

use crate::{
    collections::{Collection, CollectionPtr},
    data_input::DataInput,
    ergo_box::ErgoBoxCandidate,
    input::UnsignedInput,
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Unsigned (inputs without proofs) transaction
#[derive(PartialEq, Debug, Clone)]
pub struct UnsignedTransaction(chain::transaction::unsigned::UnsignedTransaction);
pub type UnsignedTransactionPtr = *mut UnsignedTransaction;
pub type ConstUnsignedTransactionPtr = *const UnsignedTransaction;

pub unsafe fn unsigned_tx_id(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
) -> Result<String, Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    Ok(Base16EncodedBytes::new(unsigned_tx.0.id().0 .0.as_ref()).into())
}

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

pub unsafe fn unsigned_tx_from_json(
    json: &str,
    unsigned_tx_out: *mut UnsignedTransactionPtr,
) -> Result<(), Error> {
    let unsigned_tx_out = mut_ptr_as_mut(unsigned_tx_out, "unsigned_tx_out")?;
    let unsigned_tx = serde_json::from_str(json)
        .map(UnsignedTransaction)
        .map_err(|_| Error::Misc("UnsignedTransaction: can't deserialize from JSON".into()))?;
    *unsigned_tx_out = Box::into_raw(Box::new(unsigned_tx));
    Ok(())
}

pub unsafe fn unsigned_tx_to_json(
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
) -> Result<String, Error> {
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    serde_json::to_string(&unsigned_tx.0)
        .map_err(|_| Error::Misc("UnsignedTransaction: can't serialize into JSON".into()))
}
