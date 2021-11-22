//! Box (aka coin, or an unspent output) is a basic concept of a UTXO-based cryptocurrency.
//! In Bitcoin, such an object is associated with some monetary value (arbitrary,
//! but with predefined precision, so we use integer arithmetic to work with the value),
//! and also a guarding script (aka proposition) to protect the box from unauthorized opening.
//!
//! In other way, a box is a state element locked by some proposition (ErgoTree).
//!
//! In Ergo, box is just a collection of registers, some with mandatory types and semantics,
//! others could be used by applications in any way.
//! We add additional fields in addition to amount and proposition~(which stored in the registers R0 and R1).
//! Namely, register R2 contains additional tokens (a sequence of pairs (token identifier, value)).
//! Register R3 contains height when block got included into the blockchain and also transaction
//! identifier and box index in the transaction outputs.
//! Registers R4-R9 are free for arbitrary usage.
//!
//! A transaction is unsealing a box. As a box can not be open twice, any further valid transaction
//! can not be linked to the same box.

use std::convert::TryFrom;

use ergo_lib::ergotree_ir::chain;

use crate::{
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Box id (32-byte digest)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxId(pub chain::ergo_box::BoxId);
pub type BoxIdPtr = *mut BoxId;
pub type ConstBoxIdPtr = *const BoxId;

pub unsafe fn box_id_from_str(box_id_str: &str, box_id_out: *mut BoxIdPtr) -> Result<(), Error> {
    let box_id_out = mut_ptr_as_mut(box_id_out, "box_id_out")?;
    let box_id = chain::ergo_box::BoxId::try_from(String::from(box_id_str))
        .map(BoxId)
        .map_err(|_| Error::Misc("BoxId: can't deserialize from string".into()))?;
    *box_id_out = Box::into_raw(Box::new(box_id));
    Ok(())
}

pub unsafe fn box_id_to_str(box_id_ptr: ConstBoxIdPtr) -> Result<String, Error> {
    let box_id_ptr = const_ptr_as_ref(box_id_ptr, "box_id_ptr")?;
    Ok(box_id_ptr.0.clone().into())
}

pub unsafe fn box_id_to_bytes(box_id_ptr: ConstBoxIdPtr, output: *mut u8) -> Result<(), Error> {
    let box_id = const_ptr_as_ref(box_id_ptr, "box_id_ptr")?;
    let src = box_id.0.as_ref();
    std::ptr::copy_nonoverlapping(src.as_ptr(), output, 32);
    Ok(())
}

/// Box value in nanoERGs with bound checks
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxValue(pub chain::ergo_box::box_value::BoxValue);
pub type BoxValuePtr = *mut BoxValue;
pub type ConstBoxValuePtr = *const BoxValue;

/// Create from i64 with bounds check
pub unsafe fn box_value_from_i64(
    amount: i64,
    box_value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let box_value_out = mut_ptr_as_mut(box_value_out, "box_value_out")?;
    let inner = chain::ergo_box::box_value::BoxValue::try_from(amount as u64)
        .map_err(|_| Error::Misc("BoxValue: can't parse from i64".into()))?;
    *box_value_out = Box::into_raw(Box::new(BoxValue(inner)));
    Ok(())
}

/// Get value as signed 64-bit long
pub unsafe fn box_value_as_i64(box_value_ptr: ConstBoxValuePtr) -> Result<i64, Error> {
    let box_value = const_ptr_as_ref(box_value_ptr, "box_value_ptr")?;
    Ok(i64::from(box_value.0))
}
