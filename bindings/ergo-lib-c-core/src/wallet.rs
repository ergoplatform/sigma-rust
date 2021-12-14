//! Wallet-like features

use ergo_lib::chain::transaction::TxIoVec;
use ergo_lib::wallet::signing::ErgoTransaction;

use crate::{
    collections::ConstCollectionPtr,
    ergo_box::ErgoBox,
    ergo_state_ctx::ConstErgoStateContextPtr,
    reduced::ConstReducedTransactionPtr,
    secret_key::SecretKey,
    transaction::{ConstUnsignedTransactionPtr, Transaction, TransactionPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// A collection of secret keys. This simplified signing by matching the secret keys to the correct inputs automatically.
pub struct Wallet(ergo_lib::wallet::Wallet);
pub type WalletPtr = *mut Wallet;
pub type ConstWalletPtr = *const Wallet;

/// Create `Wallet` instance loading secret key from mnemonic Returns Err if a DlogSecretKey cannot be
/// parsed from the provided phrase
pub unsafe fn wallet_from_mnemonic(
    mnemonic_phrase: &str,
    mnemonic_pass: &str,
    wallet_out: *mut WalletPtr,
) -> Result<(), Error> {
    let wallet_out = mut_ptr_as_mut(wallet_out, "wallet_out")?;
    if let Some(wallet_inner) =
        ergo_lib::wallet::Wallet::from_mnemonic(mnemonic_phrase, mnemonic_pass)
    {
        *wallet_out = Box::into_raw(Box::new(Wallet(wallet_inner)));
        Ok(())
    } else {
        Err(Error::Misc(
            "Wallet.from_mnemonic: DlogSecretKey can't be parsed from the provided phrase".into(),
        ))
    }
}

/// Create `Wallet` from secrets
pub unsafe fn wallet_from_secrets(
    secret_keys_ptr: ConstCollectionPtr<SecretKey>,
    wallet_out: *mut WalletPtr,
) -> Result<(), Error> {
    let secret_keys = const_ptr_as_ref(secret_keys_ptr, "secret_keys_ptr")?;
    let wallet_out = mut_ptr_as_mut(wallet_out, "wallet_out")?;
    *wallet_out = Box::into_raw(Box::new(Wallet(ergo_lib::wallet::Wallet::from_secrets(
        secret_keys.0.clone().into_iter().map(|s| s.0).collect(),
    ))));
    Ok(())
}

/// Signs a transaction
pub unsafe fn wallet_sign_transaction(
    wallet_ptr: ConstWalletPtr,
    state_context_ptr: ConstErgoStateContextPtr,
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    transaction_out: *mut TransactionPtr,
) -> Result<(), Error> {
    let wallet = const_ptr_as_ref(wallet_ptr, "wallet_ptr")?;
    let state_context = const_ptr_as_ref(state_context_ptr, "state_context_ptr")?;
    let unsigned_tx = const_ptr_as_ref(unsigned_tx_ptr, "unsigned_tx_ptr")?;
    let boxes_to_spend = const_ptr_as_ref(boxes_to_spend_ptr, "boxes_to_spend_ptr")?;
    let data_boxes = const_ptr_as_ref(data_boxes_ptr, "data_boxes_ptr")?;
    let transaction_out = mut_ptr_as_mut(transaction_out, "transaction_out")?;
    let boxes_to_spend =
        TxIoVec::from_vec(boxes_to_spend.0.clone().into_iter().map(|b| b.0).collect())?;
    let data_boxes = {
        let d: Vec<_> = data_boxes.0.clone().into_iter().map(|b| b.0).collect();
        if d.is_empty() {
            None
        } else {
            Some(TxIoVec::from_vec(d)?)
        }
    };
    let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
        unsigned_tx.0.clone(),
        boxes_to_spend,
        data_boxes,
    )?;
    let tx = wallet
        .0
        .sign_transaction(tx_context, &state_context.0, None)?;
    *transaction_out = Box::into_raw(Box::new(Transaction(tx)));
    Ok(())
}

/// Signs a reduced transaction (generating proofs for inputs)
pub unsafe fn wallet_sign_reduced_transaction(
    wallet_ptr: ConstWalletPtr,
    reduced_tx_ptr: ConstReducedTransactionPtr,
    transaction_out: *mut TransactionPtr,
) -> Result<(), Error> {
    let wallet = const_ptr_as_ref(wallet_ptr, "wallet_ptr")?;
    let reduced_tx = const_ptr_as_ref(reduced_tx_ptr, "reduced_tx_ptr")?;
    let transaction_out = mut_ptr_as_mut(transaction_out, "transaction_out")?;
    let tx = wallet
        .0
        .sign_reduced_transaction(reduced_tx.0.clone(), None)
        .map(Transaction)?;
    *transaction_out = Box::into_raw(Box::new(tx));
    Ok(())
}
