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

use std::convert::{TryFrom, TryInto};

use ergo_lib::ergotree_ir::chain::{self, ergo_box::NonMandatoryRegisters};

use crate::{
    collections::{CollectionPtr, ConstCollectionPtr},
    constant::{Constant, ConstantPtr},
    contract::ConstContractPtr,
    ergo_tree::{ErgoTree, ErgoTreePtr},
    json::ErgoBoxJsonEip12,
    token::Token,
    transaction::ConstTxIdPtr,
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Box id (32-byte digest)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxId(pub chain::ergo_box::BoxId);
pub type BoxIdPtr = *mut BoxId;
pub type ConstBoxIdPtr = *const BoxId;

/// Parse box id (32 byte digest) from base16-encoded string
pub unsafe fn box_id_from_str(box_id_str: &str, box_id_out: *mut BoxIdPtr) -> Result<(), Error> {
    let box_id_out = mut_ptr_as_mut(box_id_out, "box_id_out")?;
    let box_id = chain::ergo_box::BoxId::try_from(String::from(box_id_str)).map(BoxId)?;
    *box_id_out = Box::into_raw(Box::new(box_id));
    Ok(())
}

/// Base16 encoded string
pub unsafe fn box_id_to_str(box_id_ptr: ConstBoxIdPtr) -> Result<String, Error> {
    let box_id_ptr = const_ptr_as_ref(box_id_ptr, "box_id_ptr")?;
    Ok(box_id_ptr.0.clone().into())
}

/// Convert to serialized bytes. Key assumption: 32 bytes have been allocated at the address
/// pointed-to by `output`.
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

/// Recommended (safe) minimal box value to use in case box size estimation is unavailable.
/// Allows box size upto 2777 bytes with current min box value per byte of 360 nanoERGs
pub unsafe fn box_value_safe_user_min(box_value_out: *mut BoxValuePtr) -> Result<(), Error> {
    let box_value_out = mut_ptr_as_mut(box_value_out, "box_value_out")?;
    *box_value_out = Box::into_raw(Box::new(BoxValue(
        chain::ergo_box::box_value::BoxValue::SAFE_USER_MIN,
    )));
    Ok(())
}

/// Number of units inside one ERGO (i.e. one ERG using nano ERG representation)
pub fn box_value_units_per_ergo() -> i64 {
    chain::ergo_box::box_value::BoxValue::UNITS_PER_ERGO as i64
}

/// Create from i64 with bounds check
pub unsafe fn box_value_from_i64(
    amount: i64,
    box_value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let box_value_out = mut_ptr_as_mut(box_value_out, "box_value_out")?;
    let inner = chain::ergo_box::box_value::BoxValue::try_from(amount as u64)?;
    *box_value_out = Box::into_raw(Box::new(BoxValue(inner)));
    Ok(())
}

/// Get value as signed 64-bit long
pub unsafe fn box_value_as_i64(box_value_ptr: ConstBoxValuePtr) -> Result<i64, Error> {
    let box_value = const_ptr_as_ref(box_value_ptr, "box_value_ptr")?;
    Ok(i64::from(box_value.0))
}

/// Create a new box value which is the sum of the arguments, returning error if value is out of
/// bounds.
pub unsafe fn box_value_sum_of(
    box_value0_ptr: ConstBoxValuePtr,
    box_value1_ptr: ConstBoxValuePtr,
    sum_of_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let box_value0 = const_ptr_as_ref(box_value0_ptr, "box_value0_ptr")?;
    let box_value1 = const_ptr_as_ref(box_value1_ptr, "box_value1_ptr")?;
    let sum_of_out = mut_ptr_as_mut(sum_of_out, "sum_of_out")?;
    let sum = box_value0
        .0
        .as_i64()
        .checked_add(box_value1.0.as_i64())
        .ok_or_else(|| Error::Misc("BoxValue.sum_of: sum of i64 overflowed".into()))?;
    let inner = chain::ergo_box::box_value::BoxValue::try_from(sum as u64)?;
    *sum_of_out = Box::into_raw(Box::new(BoxValue(inner)));
    Ok(())
}

/// Contains the same fields as `ErgoBox`, except for transaction id and index, that will be
/// calculated after full transaction formation.  Use `ErgoBoxCandidateBuilder` to create an
/// instance.
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
    tokens_out: *mut CollectionPtr<Token>,
) -> Result<(), Error> {
    let candidate = const_ptr_as_ref(ergo_box_candidate_ptr, "ergo_box_candidate_ptr")?;
    let tokens_out = mut_ptr_as_mut(tokens_out, "tokens_out")?;
    *tokens_out = Box::into_raw(Box::new(candidate.0.tokens.clone().into()));
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

/// Ergo box, that is taking part in some transaction on the chain Differs with [`ErgoBoxCandidate`]
/// by added transaction id and an index in the input of that transaction
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBox(pub(crate) chain::ergo_box::ErgoBox);
pub type ErgoBoxPtr = *mut ErgoBox;
pub type ConstErgoBoxPtr = *const ErgoBox;

/// Make a new box with:
/// `value` - amount of money associated with the box
/// `contract` - guarding contract, which should be evaluated to true in order
/// to open(spend) this box
/// `creation_height` - height when a transaction containing the box is created.
/// `tx_id` - transaction id in which this box was "created" (participated in outputs)
/// `index` - index (in outputs) in the transaction
pub unsafe fn ergo_box_new(
    value_ptr: ConstBoxValuePtr,
    creation_height: u32,
    contract_ptr: ConstContractPtr,
    tx_id_ptr: ConstTxIdPtr,
    index: u16,
    tokens_ptr: ConstCollectionPtr<Token>,
    ergo_box_out: *mut ErgoBoxPtr,
) -> Result<(), Error> {
    let value = const_ptr_as_ref(value_ptr, "value_ptr")?;
    let contract = const_ptr_as_ref(contract_ptr, "contract_ptr")?;
    let tx_id = const_ptr_as_ref(tx_id_ptr, "tx_id_ptr")?;
    let tokens = const_ptr_as_ref(tokens_ptr, "tokens_ptr")?;
    let ergo_box_out = mut_ptr_as_mut(ergo_box_out, "ergo_box_out")?;

    let chain_ergo_tree = contract.0.ergo_tree();
    let ergo_box = chain::ergo_box::ErgoBox::new(
        value.0,
        chain_ergo_tree,
        tokens.try_into().unwrap(),
        NonMandatoryRegisters::empty(),
        creation_height,
        tx_id.0.clone(),
        index,
    )
    .map(ErgoBox)?;
    *ergo_box_out = Box::into_raw(Box::new(ergo_box));
    Ok(())
}

/// Get box id
pub unsafe fn ergo_box_box_id(
    ergo_box_ptr: ConstErgoBoxPtr,
    box_id_out: *mut BoxIdPtr,
) -> Result<(), Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    let box_id_out = mut_ptr_as_mut(box_id_out, "box_id_out")?;
    *box_id_out = Box::into_raw(Box::new(BoxId(ergo_box.0.box_id())));
    Ok(())
}

/// Get box creation height
pub unsafe fn ergo_box_creation_height(ergo_box_ptr: ConstErgoBoxPtr) -> Result<u32, Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    Ok(ergo_box.0.creation_height)
}

/// Get tokens for box
pub unsafe fn ergo_box_tokens(
    ergo_box_ptr: ConstErgoBoxPtr,
    tokens_out: *mut CollectionPtr<Token>,
) -> Result<(), Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    let tokens_out = mut_ptr_as_mut(tokens_out, "tokens_out")?;
    *tokens_out = Box::into_raw(Box::new(ergo_box.0.tokens.clone().into()));
    Ok(())
}

/// Get ergo tree for box
pub unsafe fn ergo_box_ergo_tree(
    ergo_box_ptr: ConstErgoBoxPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) -> Result<(), Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    let ergo_tree_out = mut_ptr_as_mut(ergo_tree_out, "ergo_tree_out")?;
    *ergo_tree_out = Box::into_raw(Box::new(ErgoTree(ergo_box.0.ergo_tree.clone())));
    Ok(())
}

/// Get box value in nanoERGs
pub unsafe fn ergo_box_value(
    ergo_box_ptr: ConstErgoBoxPtr,
    box_value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    let box_value_out = mut_ptr_as_mut(box_value_out, "box_value_out")?;
    *box_value_out = Box::into_raw(Box::new(BoxValue(ergo_box.0.value)));
    Ok(())
}

/// Returns value (ErgoTree constant) stored in the register or None if the register is empty
pub unsafe fn ergo_box_register_value(
    ergo_box_ptr: ConstErgoBoxPtr,
    register_id: NonMandatoryRegisterId,
    constant_out: *mut ConstantPtr,
) -> Result<bool, Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    if let Some(c) = ergo_box.0.additional_registers.get(register_id.into()) {
        *constant_out = Box::into_raw(Box::new(Constant(c.clone())));
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parse from JSON.  Supports Ergo Node/Explorer API and box values and token amount encoded as
/// strings
pub unsafe fn ergo_box_from_json(json: &str, ergo_box_out: *mut ErgoBoxPtr) -> Result<(), Error> {
    let ergo_box_out = mut_ptr_as_mut(ergo_box_out, "ergo_box_out")?;
    let unsigned_tx = serde_json::from_str(json).map(ErgoBox)?;
    *ergo_box_out = Box::into_raw(Box::new(unsigned_tx));
    Ok(())
}

/// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
pub unsafe fn ergo_box_to_json(ergo_box_ptr: ConstErgoBoxPtr) -> Result<String, Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    let s = serde_json::to_string(&ergo_box.0)?;
    Ok(s)
}

/// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
pub unsafe fn ergo_box_to_json_eip12(ergo_box_ptr: ConstErgoBoxPtr) -> Result<String, Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    let box_dapp: ErgoBoxJsonEip12 = ergo_box.0.clone().into();
    let s = serde_json::to_string(&box_dapp)?;
    Ok(s)
}

/// Pair of <value, tokens> for a box
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxAssetsData(pub(crate) ergo_lib::wallet::box_selector::ErgoBoxAssetsData);
pub type ErgoBoxAssetsDataPtr = *mut ErgoBoxAssetsData;
pub type ConstErgoBoxAssetsDataPtr = *const ErgoBoxAssetsData;

/// Create new instance
pub unsafe fn ergo_box_assets_data_new(
    value_ptr: ConstBoxValuePtr,
    tokens_ptr: ConstCollectionPtr<Token>,
    ergo_box_assets_data_out: *mut ErgoBoxAssetsDataPtr,
) -> Result<(), Error> {
    let value = const_ptr_as_ref(value_ptr, "value_ptr")?;
    let tokens = const_ptr_as_ref(tokens_ptr, "tokens_ptr")?;
    let ergo_box_assets_data_out =
        mut_ptr_as_mut(ergo_box_assets_data_out, "ergo_box_assets_data_out")?;
    *ergo_box_assets_data_out = Box::into_raw(Box::new(ErgoBoxAssetsData(
        ergo_lib::wallet::box_selector::ErgoBoxAssetsData {
            value: value.0,
            tokens: tokens.try_into()?,
        },
    )));
    Ok(())
}

/// Value part of the box
pub unsafe fn ergo_box_assets_data_value(
    ergo_box_assets_data_ptr: ConstErgoBoxAssetsDataPtr,
    value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let ergo_box_assets_data =
        const_ptr_as_ref(ergo_box_assets_data_ptr, "ergo_box_assets_data_ptr")?;
    let value_out = mut_ptr_as_mut(value_out, "value_out")?;

    *value_out = Box::into_raw(Box::new(BoxValue(ergo_box_assets_data.0.value)));
    Ok(())
}

/// Tokens part of the box
pub unsafe fn ergo_box_assets_data_tokens(
    ergo_box_assets_data_ptr: ConstErgoBoxAssetsDataPtr,
    tokens_out: *mut CollectionPtr<Token>,
) -> Result<(), Error> {
    let ergo_box_assets_data =
        const_ptr_as_ref(ergo_box_assets_data_ptr, "ergo_box_assets_data_ptr")?;
    let tokens_out = mut_ptr_as_mut(tokens_out, "tokens_out")?;
    *tokens_out = Box::into_raw(Box::new(ergo_box_assets_data.0.tokens.clone().into()));
    Ok(())
}

/// Type for representing box registers R4 - R9
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

impl From<chain::ergo_box::NonMandatoryRegisterId> for NonMandatoryRegisterId {
    fn from(v: chain::ergo_box::NonMandatoryRegisterId) -> Self {
        use NonMandatoryRegisterId::*;
        match v {
            chain::ergo_box::NonMandatoryRegisterId::R4 => R4,
            chain::ergo_box::NonMandatoryRegisterId::R5 => R5,
            chain::ergo_box::NonMandatoryRegisterId::R6 => R6,
            chain::ergo_box::NonMandatoryRegisterId::R7 => R7,
            chain::ergo_box::NonMandatoryRegisterId::R8 => R8,
            chain::ergo_box::NonMandatoryRegisterId::R9 => R9,
        }
    }
}
