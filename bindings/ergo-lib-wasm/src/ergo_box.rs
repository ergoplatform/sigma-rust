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

use chain::ergo_box::register::NonMandatoryRegisters;
use ergo_lib::chain;
use ergo_lib::chain::ergo_box::register::NonMandatoryRegisterId;
use wasm_bindgen::prelude::*;

use crate::ast::Constant;
use crate::{contract::Contract, transaction::TxId};

pub mod box_builder;

/// ErgoBox candidate not yet included in any transaction on the chain
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxCandidate(chain::ergo_box::ErgoBoxCandidate);

#[wasm_bindgen]
impl ErgoBoxCandidate {
    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r4(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R4)
            .cloned()
            .map(Constant::from)
    }
    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r5(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R5)
            .cloned()
            .map(Constant::from)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r6(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R6)
            .cloned()
            .map(Constant::from)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r7(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R7)
            .cloned()
            .map(Constant::from)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r9(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R9)
            .cloned()
            .map(Constant::from)
    }
}

impl Into<chain::ergo_box::ErgoBoxCandidate> for ErgoBoxCandidate {
    fn into(self) -> chain::ergo_box::ErgoBoxCandidate {
        self.0
    }
}

/// Ergo box, that is taking part in some transaction on the chain
/// Differs with [`ErgoBoxCandidate`] by added transaction id and an index in the input of that transaction
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBox(chain::ergo_box::ErgoBox);

#[wasm_bindgen]
impl ErgoBox {
    /// make a new box with:
    /// `value` - amount of money associated with the box
    /// `contract` - guarding contract([`Contract`]), which should be evaluated to true in order
    /// to open(spend) this box
    /// `creation_height` - height when a transaction containing the box is created.
    /// `tx_id` - transaction id in which this box was "created" (participated in outputs)
    /// `index` - index (in outputs) in the transaction
    #[wasm_bindgen(constructor)]
    pub fn new(
        value: &BoxValue,
        creation_height: u32,
        contract: &Contract,
        tx_id: &TxId,
        index: u16,
    ) -> ErgoBox {
        let chain_contract: chain::contract::Contract = contract.clone().into();
        let b = chain::ergo_box::ErgoBox::new(
            value.0,
            chain_contract.ergo_tree(),
            vec![],
            NonMandatoryRegisters::empty(),
            creation_height,
            tx_id.clone().into(),
            index,
        );
        ErgoBox(b)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r4(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R4)
            .cloned()
            .map(Constant::from)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r5(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R5)
            .cloned()
            .map(Constant::from)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r6(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R6)
            .cloned()
            .map(Constant::from)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r7(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R7)
            .cloned()
            .map(Constant::from)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r8(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R8)
            .cloned()
            .map(Constant::from)
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn r9(&self) -> Option<Constant> {
        self.0
            .additional_registers
            .get(NonMandatoryRegisterId::R9)
            .cloned()
            .map(Constant::from)
    }

    // JSON representation
    // pub fn to_json(&self) -> Result<JsValue, JsValue> {
    //     JsValue::from_serde(&self.0).map_err(|e| JsValue::from_str(&format!("{}", e)))
    // }
}

impl Into<chain::ergo_box::ErgoBox> for ErgoBox {
    fn into(self) -> chain::ergo_box::ErgoBox {
        self.0
    }
}

/// Box value with with bound checks
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxValue(chain::ergo_box::box_value::BoxValue);

#[wasm_bindgen]
impl BoxValue {
    /// Recommended (safe) minimal box value to use in case box size estimation is unavailable.
    /// Allows box size upto 2777 bytes with current min box value per byte of 360 nanoERGs
    #[allow(non_snake_case)]
    pub fn SAFE_USER_MIN() -> BoxValue {
        BoxValue(chain::ergo_box::box_value::BoxValue::SAFE_USER_MIN)
    }

    /// Create from u32 with bounds check
    pub fn from_u32(v: u32) -> Result<BoxValue, JsValue> {
        Ok(BoxValue(
            chain::ergo_box::box_value::BoxValue::try_from(v as u64)
                .map_err(|e| JsValue::from_str(&format!("{}", e)))?,
        ))
    }
}

impl From<BoxValue> for chain::ergo_box::box_value::BoxValue {
    fn from(v: BoxValue) -> Self {
        v.0
    }
}

impl From<chain::ergo_box::box_value::BoxValue> for BoxValue {
    fn from(v: chain::ergo_box::box_value::BoxValue) -> Self {
        BoxValue(v)
    }
}
