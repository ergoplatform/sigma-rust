//! Extended Public Key functionality

use derive_more::{From, Into};

use ergo_lib::ergotree_ir::chain::address::Address as InnerAddress;
use ergo_lib::wallet::derivation_path::ChildIndexNormal;
use ergo_lib::wallet::ext_pub_key::{ExtPubKey as InnerExtPubKey, PubKeyBytes};
use ergo_lib::ArrLength;

use crate::address::{Address, AddressPtr};
use crate::derivation_path::ConstDerivationPathPtr;
use crate::util::{const_ptr_as_ref, mut_ptr_as_mut};
use crate::Error;

#[derive(From, Into)]
pub struct ExtPubKey(pub InnerExtPubKey);
pub type ExtPubKeyPtr = *mut ExtPubKey;
pub type ConstExtPubKeyPtr = *const ExtPubKey;

/// Create ExtPubKey from public key bytes, chain code and derivation path
/// public_key_bytes needs to be the length of PubKeyBytes::LEN (33 bytes)
/// chain_code needs to be the length of ChainCode::LEN (32 bytes)
pub unsafe fn ext_pub_key_new(
    public_key_bytes: *const u8,
    chain_code: *const u8,
    derivation_path_ptr: ConstDerivationPathPtr,
    ext_pub_key_out: *mut ExtPubKeyPtr,
) -> Result<(), Error> {
    let ext_pub_key_out = mut_ptr_as_mut(ext_pub_key_out, "ext_pub_key_out")?;
    let derivation_path = const_ptr_as_ref(derivation_path_ptr, "derivation_path_ptr")?;
    let secret_key_bytes = std::slice::from_raw_parts(public_key_bytes, PubKeyBytes::LEN);
    let chain_code =
        std::slice::from_raw_parts(chain_code, ergo_lib::wallet::ext_pub_key::ChainCode::LEN);
    let key = InnerExtPubKey::new(
        secret_key_bytes.try_into().map_err(Error::misc)?,
        chain_code.try_into().map_err(Error::misc)?,
        derivation_path.0.clone(),
    )
    .map_err(Error::misc)?;
    *ext_pub_key_out = Box::into_raw(Box::new(ExtPubKey(key)));
    Ok(())
}

/// Derive a new extended public key from the provided index
/// The index is in the form of soft or hardened indices
/// For example: 4 or 4' respectively
pub unsafe fn ext_pub_key_child(
    derive_from_key_ptr: ConstExtPubKeyPtr,
    child_index: u32,
    ext_pub_key_out: *mut ExtPubKeyPtr,
) -> Result<(), Error> {
    let ext_pub_key = const_ptr_as_ref(derive_from_key_ptr, "derive_from_key_ptr")?;
    let ext_pub_key_out = mut_ptr_as_mut(ext_pub_key_out, "ext_pub_key_out")?;
    let index = ChildIndexNormal::normal(child_index).map_err(Error::misc)?;
    let key = ext_pub_key.0.child(index).map_err(Error::misc)?;
    *ext_pub_key_out = Box::into_raw(Box::new(ExtPubKey(key)));
    Ok(())
}

/// Derive a new extended public key from the derivation path
pub unsafe fn ext_pub_key_derive(
    ext_pub_key_ptr: ConstExtPubKeyPtr,
    derivation_path_ptr: ConstDerivationPathPtr,
    ext_pub_key_out: *mut ExtPubKeyPtr,
) -> Result<(), Error> {
    let ext_pub_key_ptr = const_ptr_as_ref(ext_pub_key_ptr, "ext_pub_key_ptr")?;
    let derivation_path = const_ptr_as_ref(derivation_path_ptr, "derivation_path_ptr")?;
    let ext_pub_key_out = mut_ptr_as_mut(ext_pub_key_out, "ext_pub_key_out")?;
    *ext_pub_key_out = Box::into_raw(Box::new(ExtPubKey(
        ext_pub_key_ptr
            .0
            .derive(derivation_path.0.clone())
            .map_err(Error::misc)?,
    )));
    Ok(())
}

/// Get address for extended public key
pub unsafe fn ext_pub_key_address(
    ext_pub_key_ptr: ConstExtPubKeyPtr,
    address_out: *mut AddressPtr,
) -> Result<(), Error> {
    let ext_pub_key_ptr = const_ptr_as_ref(ext_pub_key_ptr, "ext_pub_key_ptr")?;
    let address_out = mut_ptr_as_mut(address_out, "address_out")?;
    let ext_pub_key: InnerExtPubKey = ext_pub_key_ptr.0.clone();
    let address = InnerAddress::from(ext_pub_key);
    *address_out = Box::into_raw(Box::new(Address(address)));
    Ok(())
}
