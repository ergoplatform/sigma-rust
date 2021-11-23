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
    constant::{Constant, ConstantPtr},
    ergo_tree::{ErgoTree, ErgoTreePtr},
    token::{Token, Tokens, TokensPtr},
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

/// ErgoBox candidate not yet included in any transaction on the chain
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxCandidate(pub(crate) chain::ergo_box::ErgoBoxCandidate);
pub type ErgoBoxCandidatePtr = *mut ErgoBoxCandidate;
pub type ConstErgoBoxCandidatePtr = *const ErgoBoxCandidate;

/// Returns value (ErgoTree constant) stored in the register or None if the register is empty
pub unsafe fn ergo_box_candidate_register_value(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    register_id: NonMandatoryRegisterId,
    constant_out: *mut ConstantPtr,
) -> Result<bool, Error> {
    let candidate = const_ptr_as_ref(ergo_box_candidate_ptr, "ergo_box_candidate_ptr")?;
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    if let Some(c) = candidate.0.additional_registers.get(register_id.into()) {
        *constant_out = Box::into_raw(Box::new(Constant(c.clone())));
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Get box creation height
pub unsafe fn ergo_box_candidate_creation_height(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
) -> Result<u32, Error> {
    let candidate = const_ptr_as_ref(ergo_box_candidate_ptr, "ergo_box_candidate_ptr")?;
    Ok(candidate.0.creation_height)
}

/// Get tokens for box
pub unsafe fn ergo_box_candidate_tokens(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    tokens_out: *mut TokensPtr,
) -> Result<(), Error> {
    let candidate = const_ptr_as_ref(ergo_box_candidate_ptr, "ergo_box_candidate_ptr")?;
    let tokens_out = mut_ptr_as_mut(tokens_out, "tokens_out")?;
    *tokens_out = Box::into_raw(Box::new(Tokens(
        candidate.0.tokens.clone().map(|bv| bv.mapped(Token)),
    )));
    Ok(())
}

/// Get ergo tree for box
pub unsafe fn ergo_box_candidate_ergo_tree(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    ergo_tree_out: *mut ErgoTreePtr,
) -> Result<(), Error> {
    let candidate = const_ptr_as_ref(ergo_box_candidate_ptr, "ergo_box_candidate_ptr")?;
    let ergo_tree_out = mut_ptr_as_mut(ergo_tree_out, "ergo_tree_out")?;
    *ergo_tree_out = Box::into_raw(Box::new(ErgoTree(candidate.0.ergo_tree.clone())));
    Ok(())
}

/// Get box value in nanoERGs
pub unsafe fn ergo_box_candidate_box_value(
    ergo_box_candidate_ptr: ConstErgoBoxCandidatePtr,
    box_value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let candidate = const_ptr_as_ref(ergo_box_candidate_ptr, "ergo_box_candidate_ptr")?;
    let box_value_out = mut_ptr_as_mut(box_value_out, "box_value_out")?;
    *box_value_out = Box::into_raw(Box::new(BoxValue(candidate.0.value)));
    Ok(())
}

/// newtype for box registers R4 - R9
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NonMandatoryRegisterId {
    /// id for R4 register
    R4 = 4,
    /// id for R5 register
    R5 = 5,
    /// id for R6 register
    R6 = 6,
    /// id for R7 register
    R7 = 7,
    /// id for R8 register
    R8 = 8,
    /// id for R9 register
    R9 = 9,
}

impl NonMandatoryRegisterId {}

impl From<NonMandatoryRegisterId> for chain::ergo_box::NonMandatoryRegisterId {
    fn from(v: NonMandatoryRegisterId) -> Self {
        use chain::ergo_box::NonMandatoryRegisterId::*;
        match v {
            NonMandatoryRegisterId::R4 => R4,
            NonMandatoryRegisterId::R5 => R5,
            NonMandatoryRegisterId::R6 => R6,
            NonMandatoryRegisterId::R7 => R7,
            NonMandatoryRegisterId::R8 => R8,
            NonMandatoryRegisterId::R9 => R9,
        }
    }
}