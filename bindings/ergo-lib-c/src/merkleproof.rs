use crate::{delete_ptr, ErrorPtr};
/// Used for verifying transactions and blocks
use ergo_lib_c_core::{merkleproof::*, Error};
use std::convert::TryFrom;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

/// Creates a new MerkleProof with given leaf data. Use ergo_merkle_proof_add_node to add levelnodes to the proof. leaf_data must be 32 bytes
#[no_mangle]
pub unsafe extern "C" fn ergo_merkle_proof_new(
    leaf_data: *const u8,
    len: usize,
    proof_out: *mut MerkleProofPtr,
) -> ErrorPtr {
    let leaf_data = std::slice::from_raw_parts(leaf_data, len);
    Error::c_api_from(merkleproof_new(leaf_data, proof_out))
}

/// Adds a new node (above the current level). Hash must be exactly 32 bytes. side represents the side node is on in the tree, 0 = Left, 1 = Right
#[no_mangle]
pub unsafe extern "C" fn ergo_merkle_proof_add_node(
    proof: MerkleProofPtr,
    hash: *const u8,
    hash_len: usize,
    side: u8,
) -> ErrorPtr {
    let hash = std::slice::from_raw_parts(hash, hash_len);
    let side = match NodeSide::try_from(side) {
        Ok(side) => side,
        Err(err) => return Error::c_api_from(Err(Error::InvalidArgument(err))),
    };
    Error::c_api_from(merkleproof_add_node(proof, hash, side))
}

/// Checks the merkleproof against the expected root_hash
#[no_mangle]
pub unsafe extern "C" fn ergo_merkle_proof_valid(
    proof: ConstMerkleProofPtr,
    root_hash: *const u8,
    len: usize,
) -> bool {
    let root_hash = std::slice::from_raw_parts(root_hash, len);
    #[allow(clippy::unwrap_used)]
    // Unwrap should be safe to use here unless the caller passes a null ptr
    merkleproof_valid(proof, root_hash).unwrap()
}

/// Checks the merkleproof against a base16 root_hash
#[no_mangle]
pub unsafe extern "C" fn ergo_merkle_proof_valid_base16(
    proof: ConstMerkleProofPtr,
    root_hash: *const c_char,
    valid: *mut bool,
) -> ErrorPtr {
    let root_hash = CStr::from_ptr(root_hash).to_string_lossy();
    let res = match merkleproof_valid_base16(proof, &root_hash) {
        Ok(res) => {
            *valid = res;
            Ok(())
        }
        Err(err) => Err(err),
    };
    Error::c_api_from(res)
}

/// Deserializes a MerkleProof from its json representation (see /blocks/{headerId}/proofFor/{txId} node api)
#[no_mangle]
pub unsafe extern "C" fn ergo_merkle_proof_from_json(
    json_str: *const c_char,
    proof_out: *mut MerkleProofPtr,
) -> ErrorPtr {
    let json_str = CStr::from_ptr(json_str).to_string_lossy();
    Error::c_api_from(merkleproof_from_json(&json_str, proof_out))
}

/// Serializes a MerkleProof to json representation
#[no_mangle]
pub unsafe extern "C" fn ergo_merkle_proof_to_json(
    proof: ConstMerkleProofPtr,
    json_str: *mut *const c_char,
) -> ErrorPtr {
    #[allow(clippy::unwrap_used)]
    let res = match merkleproof_to_json(proof) {
        Ok(s) => {
            *json_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(err) => Err(err),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_merkle_proof_delete(proof: MerkleProofPtr) {
    delete_ptr(proof)
}
