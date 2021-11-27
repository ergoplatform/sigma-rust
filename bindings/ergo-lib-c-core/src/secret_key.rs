//! Secret key
use ergo_lib::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergo_lib::wallet;
use std::convert::TryInto;

use crate::util::const_ptr_as_ref;
use crate::Error;

#[derive(PartialEq, Debug, Clone)]
pub struct SecretKey(wallet::secret_key::SecretKey);
pub type SecretKeyPtr = *mut SecretKey;
pub type ConstSecretKeyPtr = *const SecretKey;

pub unsafe fn secret_key_from_bytes(
    bytes_ptr: *const u8,
    len: usize,
    secret_key_out: *mut SecretKeyPtr,
) -> Result<(), Error> {
    if bytes_ptr.is_null() {
        return Err(Error::Misc("bytes_ptr is null".into()));
    }
    let bytes = std::slice::from_raw_parts(bytes_ptr, len);
    let sized_bytes: &[u8; DlogProverInput::SIZE_BYTES] = bytes
        .try_into()
        .map_err(|_| Error::Misc("bytes_ptr is not 32 bytes".into()))?;
    if let Some(k) = wallet::secret_key::SecretKey::dlog_from_bytes(sized_bytes).map(SecretKey) {
        *secret_key_out = Box::into_raw(Box::new(k));
        Ok(())
    } else {
        Err(Error::Misc("failed to parse scalar".into()))
    }
}

pub unsafe fn secret_key_generate_random(secret_key_out: *mut SecretKeyPtr) -> Result<(), Error> {
    *secret_key_out = Box::into_raw(Box::new(SecretKey(
        wallet::secret_key::SecretKey::random_dlog(),
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

pub fn secret_key_delete(ptr: SecretKeyPtr) {
    if !ptr.is_null() {
        let boxed = unsafe { Box::from_raw(ptr) };
        std::mem::drop(boxed);
    }
}
