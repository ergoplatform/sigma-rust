//! Extended Secret Key functionality

use std::convert::TryInto;

use crate::util::const_ptr_as_ref;
use crate::{util::mut_ptr_as_mut, Error};
use derive_more::{From, Into};
use ergo_lib::wallet::derivation_path::{ChildIndex, DerivationPath};
use ergo_lib::wallet::ext_secret_key::{
    ChainCode, ExtSecretKey as InnerExtSecretKey, SecretKeyBytes,
};
use ergo_lib::wallet::mnemonic::MnemonicSeed;
use ergo_lib::ArrLength;

#[derive(From, Into)]
pub struct ExtSecretKey(InnerExtSecretKey);
pub type ExtSecretKeyPtr = *mut ExtSecretKey;
pub type ConstExtSecretKeyPtr = *const ExtSecretKey;

/// Create ExtSecretKey from secret key bytes, chain code and derivation path
/// Derivation path should be a string in the form of: m/44/429/acc'/0/addr
pub unsafe fn ext_secret_key_new(
    secret_key_bytes: *const u8,
    chain_code: *const u8,
    derivation_path: &str,
    ext_secret_key_out: *mut ExtSecretKeyPtr,
) -> Result<(), Error> {
    let ext_secret_key_out = mut_ptr_as_mut(ext_secret_key_out, "ext_secret_key_out")?;
    let derivation_path: DerivationPath = derivation_path.parse().map_err(Error::misc)?;
    let secret_key_bytes = std::slice::from_raw_parts(secret_key_bytes, SecretKeyBytes::LEN);
    let chain_code = std::slice::from_raw_parts(chain_code, ChainCode::LEN);
    let key = InnerExtSecretKey::new(
        secret_key_bytes.try_into().map_err(Error::misc)?,
        chain_code.try_into().map_err(Error::misc)?,
        derivation_path,
    )
    .map_err(Error::misc)?;
    *ext_secret_key_out = Box::into_raw(Box::new(ExtSecretKey(key)));
    Ok(())
}

/// Derive root extended secret key
pub unsafe fn ext_secret_key_derive_master(
    seed: *const u8,
    ext_secret_key_out: *mut ExtSecretKeyPtr,
) -> Result<(), Error> {
    let ext_secret_key_out = mut_ptr_as_mut(ext_secret_key_out, "ext_secret_key_out")?;
    let seed = std::slice::from_raw_parts(seed, MnemonicSeed::LEN);
    let key = InnerExtSecretKey::derive_master(seed.try_into().map_err(Error::misc)?)
        .map_err(Error::misc)?;
    *ext_secret_key_out = Box::into_raw(Box::new(ExtSecretKey(key)));
    Ok(())
}

/// Derive a new extended secret key from the provided index
/// The index is in the form of soft or hardened indices
/// For example: 4 or 4' respectively
pub unsafe fn ext_secret_key_derive(
    key_ptr: ConstExtSecretKeyPtr,
    index: &str,
    ext_secret_key_out: *mut ExtSecretKeyPtr,
) -> Result<(), Error> {
    let ext_secret_key = const_ptr_as_ref(key_ptr, "key_ptr")?;
    let ext_secret_key_out = mut_ptr_as_mut(ext_secret_key_out, "ext_secret_key_out")?;
    let index = index.parse::<ChildIndex>().map_err(Error::misc)?;
    let key = ext_secret_key.0.derive(index).map_err(Error::misc)?;
    *ext_secret_key_out = Box::into_raw(Box::new(ExtSecretKey(key)));
    Ok(())
}
