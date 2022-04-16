use crate::{delete_ptr, ErrorPtr};
use ergo_lib_c_core::{batchmerkleproof::*, Error};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_batch_merkle_proof_valid(
    proof: ConstBatchMerkleProofPtr,
    root_hash: *const u8,
    len: usize,
) -> bool {
    let root_hash = std::slice::from_raw_parts(root_hash, len);
    // Unwrap should be safe to use here unless the caller passes a null ptr (undefined)
    #[allow(clippy::unwrap_used)]
    batchmerkleproof_valid(proof, root_hash).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_batch_merkle_proof_from_json(
    json_str: *const c_char,
    proof_out: *mut BatchMerkleProofPtr,
) -> ErrorPtr {
    let json_str = CStr::from_ptr(json_str).to_string_lossy();
    Error::c_api_from(batchmerkleproof_from_json(&json_str, proof_out))
}

/// Serializes a BatchMerkleProof to json representation
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_batch_merkle_proof_to_json(
    proof: ConstBatchMerkleProofPtr,
    json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    // Unwrap is safe, CString::new only errors if the argument contains a 0 byte
    let res = match batchmerkleproof_to_json(proof) {
        Ok(s) => {
            *json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(err) => Err(err),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_batch_merkle_proof_delete(ptr: BatchMerkleProofPtr) {
    delete_ptr(ptr)
}
