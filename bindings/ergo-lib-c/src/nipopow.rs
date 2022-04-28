//! Bindings for NiPoPow
use paste::paste;

use ergo_lib_c_core::{
    batchmerkleproof::BatchMerkleProofPtr,
    block_header::{BlockHeader, BlockHeaderPtr, ConstBlockIdPtr},
    collections::CollectionPtr,
    nipopow::{
        nipopow_proof_from_json, nipopow_proof_is_better_than, nipopow_proof_to_json,
        nipopow_verifier_best_chain, nipopow_verifier_new, nipopow_verifier_process,
        popow_header_check_interlinks_proof, popow_header_from_json, popow_header_get_header,
        popow_header_get_interlinks, popow_header_get_interlinks_proof, popow_header_to_json,
        ConstNipopowProofPtr, ConstNipopowVerifierPtr, ConstPoPowHeaderPtr, NipopowProofPtr,
        NipopowVerifierPtr, PoPowHeaderPtr,
    },
    Error,
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::BlockIdsPtr;
use crate::{delete_ptr, ErrorPtr, ReturnBool};

/// Implementation of the ≥ algorithm from [`KMZ17`], see Algorithm 4
///
/// [`KMZ17`]: https://fc20.ifca.ai/preproceedings/74.pdf
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_nipopow_proof_is_better_than(
    nipopow_proof_ptr: ConstNipopowProofPtr,
    other_nipopow_proof_ptr: ConstNipopowProofPtr,
) -> ReturnBool {
    match nipopow_proof_is_better_than(nipopow_proof_ptr, other_nipopow_proof_ptr) {
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

/// Parse from JSON.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_nipopow_proof_from_json(
    json_str: *const c_char,
    nipopow_proof_out: *mut NipopowProofPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = nipopow_proof_from_json(&json, nipopow_proof_out);
    Error::c_api_from(res)
}

/// JSON representation as text
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_nipopow_proof_to_json(
    nipopow_proof_ptr: ConstNipopowProofPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match nipopow_proof_to_json(nipopow_proof_ptr) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

/// Delete `NipopowProof`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_nipopow_proof_delete(ptr: NipopowProofPtr) {
    delete_ptr(ptr)
}

/// Create new `NipopowVerifier` instance
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_nipopow_verifier_new(
    genesis_block_id_ptr: ConstBlockIdPtr,
    nipopow_verifier_out: *mut NipopowVerifierPtr,
) {
    #[allow(clippy::unwrap_used)]
    nipopow_verifier_new(genesis_block_id_ptr, nipopow_verifier_out).unwrap();
}

/// Returns chain of `BlockHeader`s from the best proof.
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_nipopow_verifier_best_chain(
    nipopow_verifier_ptr: ConstNipopowVerifierPtr,
    best_chain_out: *mut CollectionPtr<BlockHeader>,
) {
    #[allow(clippy::unwrap_used)]
    nipopow_verifier_best_chain(nipopow_verifier_ptr, best_chain_out).unwrap();
}

/// Process given proof
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_nipopow_verifier_process(
    nipopow_verifier_ptr: NipopowVerifierPtr,
    nipopow_proof_ptr: ConstNipopowProofPtr,
) -> ErrorPtr {
    let res = nipopow_verifier_process(nipopow_verifier_ptr, nipopow_proof_ptr);
    Error::c_api_from(res)
}

/// Delete `NipopowVerifier`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_nipopow_verifier_delete(ptr: NipopowVerifierPtr) {
    delete_ptr(ptr)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_popow_header_from_json(
    json_str: *const c_char,
    header_out: *mut PoPowHeaderPtr,
) -> ErrorPtr {
    let json = CStr::from_ptr(json_str).to_string_lossy();
    let res = popow_header_from_json(&json, header_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_popow_header_to_json(
    header: ConstPoPowHeaderPtr,
    _json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match popow_header_to_json(header) {
        Ok(s) => {
            *_json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_popow_header_get_interlinks(
    popow_header: ConstPoPowHeaderPtr,
    interlinks_out: *mut BlockIdsPtr,
) -> ErrorPtr {
    Error::c_api_from(popow_header_get_interlinks(popow_header, interlinks_out))
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_popow_header_get_header(
    popow_header: ConstPoPowHeaderPtr,
    header_out: *mut BlockHeaderPtr,
) -> ErrorPtr {
    Error::c_api_from(popow_header_get_header(popow_header, header_out))
}
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_popow_header_get_interlinks_proof(
    popow_header: ConstPoPowHeaderPtr,
    header_out: *mut BatchMerkleProofPtr,
) -> ErrorPtr {
    Error::c_api_from(popow_header_get_interlinks_proof(popow_header, header_out))
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_popow_header_check_interlinks_proof(
    popow_header_ptr: ConstPoPowHeaderPtr,
) -> bool {
    #[allow(clippy::unwrap_used)]
    popow_header_check_interlinks_proof(popow_header_ptr).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_popow_header_delete(ptr: PoPowHeaderPtr) {
    delete_ptr(ptr)
}

make_ffi_eq!(PoPowHeader);
