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

use chain::ergo_box::NonMandatoryRegisters;
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

use crate::ast::Constant;
use crate::ergo_tree::ErgoTree;
use crate::error_conversion::to_js;
use crate::token::Tokens;
use crate::utils::I64;
use crate::{contract::Contract, transaction::TxId};

extern crate derive_more;
use derive_more::{From, Into};

pub mod box_builder;

/// Box id (32-byte digest)
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct BoxId(chain::ergo_box::BoxId);

#[wasm_bindgen]
impl BoxId {
    /// Parse box id (32 byte digest) from base16-encoded string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(box_id_str: String) -> Result<BoxId, JsValue> {
        chain::ergo_box::BoxId::try_from(box_id_str)
            .map(BoxId)
            .map_err(to_js)
    }

    /// Base16 encoded string
    pub fn to_str(&self) -> String {
        self.0.clone().into()
    }
}

/// ErgoBox candidate not yet included in any transaction on the chain
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoBoxCandidate(chain::ergo_box::ErgoBoxCandidate);

#[wasm_bindgen]
impl ErgoBoxCandidate {
    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn register_value(&self, register_id: NonMandatoryRegisterId) -> Option<Constant> {
        self.0
            .additional_registers
            .get(register_id.into())
            .cloned()
            .map(Constant::from)
    }

    /// Get box creation height
    pub fn creation_height(&self) -> u32 {
        self.0.creation_height
    }

    /// Get tokens for box
    pub fn tokens(&self) -> Tokens {
        self.0.tokens.clone().into()
    }

    /// Get ergo tree for box
    pub fn ergo_tree(&self) -> ErgoTree {
        self.0.ergo_tree.clone().into()
    }

    /// Get box value in nanoERGs
    pub fn value(&self) -> BoxValue {
        self.0.value.into()
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
        tokens: &Tokens,
    ) -> Result<ErgoBox, JsValue> {
        let chain_contract: chain::contract::Contract = contract.clone().into();
        let b = chain::ergo_box::ErgoBox::new(
            value.0,
            chain_contract.ergo_tree(),
            tokens.clone().into(),
            NonMandatoryRegisters::empty(),
            creation_height,
            tx_id.clone().into(),
            index,
        )
        .map_err(to_js)?;
        Ok(ErgoBox(b))
    }

    /// Get box id
    pub fn box_id(&self) -> BoxId {
        self.0.box_id().into()
    }

    /// Get box creation height
    pub fn creation_height(&self) -> u32 {
        self.0.creation_height
    }

    /// Get tokens for box
    pub fn tokens(&self) -> Tokens {
        self.0.tokens.clone().into()
    }

    /// Get ergo tree for box
    pub fn ergo_tree(&self) -> ErgoTree {
        self.0.ergo_tree.clone().into()
    }

    /// Get box value in nanoERGs
    pub fn value(&self) -> BoxValue {
        self.0.value.into()
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty
    pub fn register_value(&self, register_id: NonMandatoryRegisterId) -> Option<Constant> {
        self.0
            .additional_registers
            .get(register_id.into())
            .cloned()
            .map(Constant::from)
    }

    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0.clone()).map_err(to_js)
    }

    /// JSON representation
    pub fn from_json(json: &str) -> Result<ErgoBox, JsValue> {
        serde_json::from_str(json).map(Self).map_err(to_js)
    }
}

impl From<ErgoBox> for chain::ergo_box::ErgoBox {
    fn from(b: ErgoBox) -> Self {
        b.0
    }
}

impl From<chain::ergo_box::ErgoBox> for ErgoBox {
    fn from(b: chain::ergo_box::ErgoBox) -> Self {
        ErgoBox(b)
    }
}

/// Box value in nanoERGs with bound checks
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxValue(pub(crate) chain::ergo_box::BoxValue);

#[wasm_bindgen]
impl BoxValue {
    /// Recommended (safe) minimal box value to use in case box size estimation is unavailable.
    /// Allows box size upto 2777 bytes with current min box value per byte of 360 nanoERGs
    #[allow(non_snake_case)]
    pub fn SAFE_USER_MIN() -> BoxValue {
        BoxValue(chain::ergo_box::BoxValue::SAFE_USER_MIN)
    }

    /// Number of units inside one ERGO (i.e. one ERG using nano ERG representation)
    #[allow(non_snake_case)]
    pub fn UNITS_PER_ERGO() -> I64 {
        (chain::ergo_box::BoxValue::UNITS_PER_ERGO as i64).into()
    }

    /// Create from i64 with bounds check
    pub fn from_i64(v: &I64) -> Result<BoxValue, JsValue> {
        Ok(BoxValue(
            chain::ergo_box::BoxValue::try_from(i64::from(v.clone()) as u64).map_err(to_js)?,
        ))
    }

    /// Get value as signed 64-bit long (I64)
    pub fn as_i64(&self) -> I64 {
        self.0.as_i64().into()
    }
}

impl From<BoxValue> for chain::ergo_box::BoxValue {
    fn from(v: BoxValue) -> Self {
        v.0
    }
}

impl From<chain::ergo_box::BoxValue> for BoxValue {
    fn from(v: chain::ergo_box::BoxValue) -> Self {
        BoxValue(v)
    }
}

/// Pair of <value, tokens> for an box
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoBoxAssetsData(chain::ergo_box::ErgoBoxAssetsData);

#[wasm_bindgen]
impl ErgoBoxAssetsData {
    /// Create empty SimpleBoxSelector
    #[wasm_bindgen(constructor)]
    pub fn new(value: &BoxValue, tokens: &Tokens) -> Self {
        ErgoBoxAssetsData(chain::ergo_box::ErgoBoxAssetsData {
            value: value.clone().into(),
            tokens: tokens.clone().into(),
        })
    }

    /// Value part of the box
    pub fn value(&self) -> BoxValue {
        self.0.value.into()
    }

    /// Tokens part of the box
    pub fn tokens(&self) -> Tokens {
        self.0.tokens.clone().into()
    }
}

/// List of asset data for a box
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxAssetsDataList(Vec<ErgoBoxAssetsData>);

#[wasm_bindgen]
impl ErgoBoxAssetsDataList {
    /// Create empty Tokens
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        ErgoBoxAssetsDataList(vec![])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> ErgoBoxAssetsData {
        self.0[index].clone()
    }

    /// Adds an elements to the collection
    pub fn add(&mut self, elem: &ErgoBoxAssetsData) {
        self.0.push(elem.clone());
    }
}

impl From<ErgoBoxAssetsDataList> for Vec<chain::ergo_box::ErgoBoxAssetsData> {
    fn from(v: ErgoBoxAssetsDataList) -> Self {
        v.0.iter().map(|i| i.0.clone()).collect()
    }
}
impl From<Vec<chain::ergo_box::ErgoBoxAssetsData>> for ErgoBoxAssetsDataList {
    fn from(v: Vec<chain::ergo_box::ErgoBoxAssetsData>) -> Self {
        let mut assets = ErgoBoxAssetsDataList::new();
        for asset in &v {
            assets.add(&ErgoBoxAssetsData(asset.clone()))
        }
        assets
    }
}

/// newtype for box registers R4 - R9
#[wasm_bindgen]
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
