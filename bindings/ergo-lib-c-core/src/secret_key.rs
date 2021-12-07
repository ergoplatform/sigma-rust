//! Secret key
use ergo_lib::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergo_lib::wallet;
use std::convert::TryInto;

use crate::address::{Address, AddressPtr};
use crate::util::{const_ptr_as_ref, mut_ptr_as_mut};
use crate::Error;

/// Secret key for the prover
#[derive(PartialEq, Debug, Clone)]
pub struct SecretKey(pub(crate) wallet::secret_key::SecretKey);
pub type SecretKeyPtr = *mut SecretKey;
pub type ConstSecretKeyPtr = *const SecretKey;

/// Parse dlog secret key from bytes (SEC-1-encoded scalar)
pub unsafe fn secret_key_from_bytes(
    bytes_ptr: *const u8,
    secret_key_out: *mut SecretKeyPtr,
) -> Result<(), Error> {
    if bytes_ptr.is_null() {
        return Err(Error::Misc("bytes_ptr is null".into()));
    }
    let bytes = std::slice::from_raw_parts(bytes_ptr, DlogProverInput::SIZE_BYTES);
    let sized_bytes: &[u8; DlogProverInput::SIZE_BYTES] = bytes.try_into()?;
    if let Some(k) = wallet::secret_key::SecretKey::dlog_from_bytes(sized_bytes).map(SecretKey) {
        *secret_key_out = Box::into_raw(Box::new(k));
        Ok(())
    } else {
        Err(Error::Misc("failed to parse scalar".into()))
    }
}

/// Generate random key
pub unsafe fn secret_key_generate_random(secret_key_out: *mut SecretKeyPtr) -> Result<(), Error> {
    *secret_key_out = Box::into_raw(Box::new(SecretKey(
        wallet::secret_key::SecretKey::random_dlog(),
    )));
    Ok(())
}

/// Address (encoded public image)
pub unsafe fn secret_key_get_address(
    secret_key_ptr: ConstSecretKeyPtr,
    address_out: *mut AddressPtr,
) -> Result<(), Error> {
    let secret_key = const_ptr_as_ref(secret_key_ptr, "secret_key_ptr")?;
    let address_out = mut_ptr_as_mut(address_out, "address_out")?;
    *address_out = Box::into_raw(Box::new(Address(
        secret_key.0.get_address_from_public_image(),
    )));
    Ok(())
}

/// Convert to serialized bytes. Key assumption: 32 bytes have been allocated at the address
/// pointed-to by `output`.
pub unsafe fn secret_key_to_bytes(
    secret_key_ptr: ConstSecretKeyPtr,
    output: *mut u8,
) -> Result<(), Error> {
    let secret_key = const_ptr_as_ref(secret_key_ptr, "secret_key_ptr")?;
    let src = secret_key.0.to_bytes();
    std::ptr::copy_nonoverlapping(src.as_ptr(), output, DlogProverInput::SIZE_BYTES);
    Ok(())
}
