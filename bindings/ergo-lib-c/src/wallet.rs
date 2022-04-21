//! Wallet-like features

use crate::ReturnBool;
use ergo_lib_c_core::{
    address::ConstAddressPtr,
    collections::ConstCollectionPtr,
    ergo_box::ErgoBox,
    ergo_state_ctx::ConstErgoStateContextPtr,
    reduced::ConstReducedTransactionPtr,
    secret_key::SecretKey,
    transaction::{
        ConstTransactionHintsBagPtr, ConstUnsignedTransactionPtr, TransactionHintsBagPtr,
        TransactionPtr,
    },
    wallet::*,
    Error, ErrorPtr,
};
use std::{ffi::CStr, os::raw::c_char};

use crate::delete_ptr;

/// Create `Wallet` instance loading secret key from mnemonic
/// Returns Err if a DlogSecretKey cannot be parsed from the provided phrase
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_from_mnemonic(
    mnemonic_phrase: *const c_char,
    mnemonic_pass: *const c_char,
    wallet_out: *mut WalletPtr,
) -> ErrorPtr {
    let mnemonic_phrase = CStr::from_ptr(mnemonic_phrase).to_string_lossy();
    let mnemonic_pass = CStr::from_ptr(mnemonic_pass).to_string_lossy();
    let res = wallet_from_mnemonic(&mnemonic_phrase, &mnemonic_pass, wallet_out);
    Error::c_api_from(res)
}

/// Create `Wallet` from secrets
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_from_secrets(
    secret_keys_ptr: ConstCollectionPtr<SecretKey>,
    wallet_out: *mut WalletPtr,
) {
    #[allow(clippy::unwrap_used)]
    wallet_from_secrets(secret_keys_ptr, wallet_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_add_secret(
    wallet_ptr: WalletPtr,
    secret_key_ptr: *mut SecretKey,
) -> ErrorPtr {
    let res = wallet_add_secret(wallet_ptr, secret_key_ptr);
    Error::c_api_from(res)
}

/// Signs a transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_sign_transaction(
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

/// Signs a multi signature transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_sign_transaction_multi(
    wallet_ptr: ConstWalletPtr,
    state_context_ptr: ConstErgoStateContextPtr,
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    tx_hints_ptr: ConstTransactionHintsBagPtr,
    transaction_out: *mut TransactionPtr,
) -> ErrorPtr {
    let res = wallet_sign_transaction_multi(
        wallet_ptr,
        state_context_ptr,
        unsigned_tx_ptr,
        boxes_to_spend_ptr,
        data_boxes_ptr,
        tx_hints_ptr,
        transaction_out,
    );
    Error::c_api_from(res)
}

/// Signs a reduced transaction (generating proofs for inputs)
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_sign_reduced_transaction(
    wallet_ptr: ConstWalletPtr,
    reduced_tx_ptr: ConstReducedTransactionPtr,
    transaction_out: *mut TransactionPtr,
) -> ErrorPtr {
    let res = wallet_sign_reduced_transaction(wallet_ptr, reduced_tx_ptr, transaction_out);
    Error::c_api_from(res)
}

/// Signs a multi signature reduced transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_sign_reduced_transaction_multi(
    wallet_ptr: ConstWalletPtr,
    reduced_tx_ptr: ConstReducedTransactionPtr,
    tx_hints_ptr: ConstTransactionHintsBagPtr,
    transaction_out: *mut TransactionPtr,
) -> ErrorPtr {
    let res = wallet_sign_reduced_transaction_multi(
        wallet_ptr,
        reduced_tx_ptr,
        tx_hints_ptr,
        transaction_out,
    );
    Error::c_api_from(res)
}

/// Generate Commitments for unsigned tx
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_generate_commitments(
    wallet_ptr: ConstWalletPtr,
    state_context_ptr: ConstErgoStateContextPtr,
    unsigned_tx_ptr: ConstUnsignedTransactionPtr,
    boxes_to_spend_ptr: ConstCollectionPtr<ErgoBox>,
    data_boxes_ptr: ConstCollectionPtr<ErgoBox>,
    transaction_hints_bag_out: *mut TransactionHintsBagPtr,
) -> ErrorPtr {
    let res = wallet_generate_commitments(
        wallet_ptr,
        state_context_ptr,
        unsigned_tx_ptr,
        boxes_to_spend_ptr,
        data_boxes_ptr,
        transaction_hints_bag_out,
    );
    Error::c_api_from(res)
}

/// Generate Commitments for reduced transaction
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_generate_commitments_for_reduced_transaction(
    wallet_ptr: ConstWalletPtr,
    reduced_tx_ptr: ConstReducedTransactionPtr,
    transaction_hints_bag_out: *mut TransactionHintsBagPtr,
) -> ErrorPtr {
    let res = wallet_generate_commitments_for_reduced_transaction(
        wallet_ptr,
        reduced_tx_ptr,
        transaction_hints_bag_out,
    );
    Error::c_api_from(res)
}

/// Sign an arbitrary message using a P2PK address
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_sign_message_using_p2pk(
    wallet_ptr: ConstWalletPtr,
    address_ptr: ConstAddressPtr,
    message_ptr: *const u8,
    message_length: usize,
    signed_message_out: *mut SignedMessagePtr,
) -> ErrorPtr {
    let res = wallet_sign_message_using_p2pk(
        wallet_ptr,
        address_ptr,
        message_ptr,
        message_length,
        signed_message_out,
    );
    Error::c_api_from(res)
}

/// Verify that the signature is presented to satisfy SigmaProp conditions.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_verify_signature(
    address_ptr: ConstAddressPtr,
    message_ptr: *const u8,
    message_length: usize,
    signed_message_ptr: ConstSignedMessagePtr,
) -> ReturnBool {
    match verify_signature(address_ptr, message_ptr, message_length, signed_message_ptr) {
        Ok(value) => ReturnBool {
            value,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnBool {
            value: false, // Just a dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

/// Drop `SignedMessage`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_signed_message_delete(ptr: SignedMessagePtr) {
    delete_ptr(ptr)
}

/// Drop `Wallet`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_wallet_delete(ptr: WalletPtr) {
    delete_ptr(ptr)
}
