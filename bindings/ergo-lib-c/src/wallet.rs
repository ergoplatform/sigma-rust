//! Wallet-like features

use std::{ffi::CStr, os::raw::c_char};

use ergo_lib_c_core::{
    collections::ConstCollectionPtr,
    ergo_box::ErgoBox,
    ergo_state_ctx::ConstErgoStateContextPtr,
    secret_key::SecretKey,
    transaction::{ConstUnsignedTransactionPtr, TransactionPtr},
    wallet::*,
    Error, ErrorPtr,
};

use crate::delete_ptr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_wallet_from_mnemonic(
    mnemonic_phrase: *const c_char,
    mnemonic_pass: *const c_char,
    wallet_out: *mut WalletPtr,
) -> ErrorPtr {
    let mnemonic_phrase = CStr::from_ptr(mnemonic_phrase).to_string_lossy();
    let mnemonic_pass = CStr::from_ptr(mnemonic_pass).to_string_lossy();
    let res = wallet_from_mnemonic(&mnemonic_phrase, &mnemonic_pass, wallet_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_wallet_from_secrets(
    secret_keys_ptr: ConstCollectionPtr<SecretKey>,
    wallet_out: *mut WalletPtr,
) {
    #[allow(clippy::unwrap_used)]
    wallet_from_secrets(secret_keys_ptr, wallet_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_wallet_sign_transaction(
    wallet_ptr: ConstWalletPtr,
    state_context_ptr: ConstErgoStateContextPtr,
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    transaction_out: *mut TransactionPtr,
) -> ErrorPtr {
    let res = wallet_sign_transaction(
        wallet_ptr,
        state_context_ptr,
        unsigned_tx_ptr,
        boxes_to_spend_ptr,
        data_boxes_ptr,
        transaction_out,
    );
    Error::c_api_from(res)
}

#[no_mangle]
pub extern "C" fn ergo_wallet_wallet_delete(ptr: WalletPtr) {
    unsafe { delete_ptr(ptr) }
}
