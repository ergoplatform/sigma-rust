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

use ergo_lib::ergotree_ir::chain;
use ergo_lib::ergotree_ir::chain::ergo_box::BoxTokens;
use ergo_lib::ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use ergo_lib::wallet::tx_builder::new_miner_fee_box;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use crate::contract::Contract;
use crate::ergo_tree::ErgoTree;
use crate::error_conversion::to_js;
use crate::json::ErgoBoxJsonEip12;
use crate::token::Tokens;
use crate::utils::I64;
use crate::{ast::Constant, transaction::TxId};

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
        self.0.into()
    }

    /// Returns byte array (32 bytes)
    pub fn as_bytes(&self) -> Uint8Array {
        Uint8Array::from(self.0.as_ref())
    }
}

/// ErgoBox candidate not yet included in any transaction on the chain
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoBoxCandidate(chain::ergo_box::ErgoBoxCandidate);

#[wasm_bindgen]
impl ErgoBoxCandidate {
    /// Create a box with miner's contract and given value
    pub fn new_miner_fee_box(
        fee_amount: &BoxValue,
        creation_height: u32,
    ) -> Result<ErgoBoxCandidate, JsValue> {
        Ok(new_miner_fee_box(fee_amount.into(), creation_height)
            .map_err(to_js)?
            .into())
    }

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty or cannot be parsed
    pub fn register_value(&self, register_id: NonMandatoryRegisterId) -> Option<Constant> {
        self.0
            .additional_registers
            .get_constant(register_id.into())
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

    /// Serialized additional register as defined in ErgoBox serialization (registers count,
    /// followed by every non-empyt register value serialized)
    pub fn serialized_additional_registers(&self) -> Result<Vec<u8>, JsValue> {
        self.0
            .additional_registers
            .sigma_serialize_bytes()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
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
        let chain_contract: ergo_lib::chain::contract::Contract = contract.clone().into();
        let b = chain::ergo_box::ErgoBox::new(
            value.0,
            chain_contract.ergo_tree(),
            tokens.clone().try_into()?,
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

    /// Get id of transaction which created the box
    pub fn tx_id(&self) -> TxId {
        self.0.transaction_id.into()
    }

    /// Index of this box in transaction outputs
    pub fn index(&self) -> u16 {
        self.0.index
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

    /// Returns value (ErgoTree constant) stored in the register or None if the register is empty or cannot be parsed
    pub fn register_value(&self, register_id: NonMandatoryRegisterId) -> Option<Constant> {
        self.0
            .additional_registers
            .get_constant(register_id.into())
            .cloned()
            .map(Constant::from)
    }

    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string_pretty(&self.0.clone())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
    /// (similar to [`Self::to_json`], but as JS object with box value and token amounts encoding as strings)
    pub fn to_js_eip12(&self) -> Result<JsValue, JsValue> {
        let box_dapp: ErgoBoxJsonEip12 = self.0.clone().into();
        <JsValue as JsValueSerdeExt>::from_serde(&box_dapp)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// parse from JSON
    /// supports Ergo Node/Explorer API and box values and token amount encoded as strings
    pub fn from_json(json: &str) -> Result<ErgoBox, JsValue> {
        serde_json::from_str(json).map(Self).map_err(to_js)
    }

    /// Serialized additional register as defined in ErgoBox serialization (registers count,
    /// followed by every non-empyt register value serialized)
    pub fn serialized_additional_registers(&self) -> Result<Vec<u8>, JsValue> {
        self.0
            .additional_registers
            .sigma_serialize_bytes()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Returns serialized bytes or fails with error if cannot be serialized
    pub fn sigma_serialize_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.0.sigma_serialize_bytes().map_err(to_js)
    }

    /// Parses ErgoBox or fails with error
    pub fn sigma_parse_bytes(data: Vec<u8>) -> Result<ErgoBox, JsValue> {
        chain::ergo_box::ErgoBox::sigma_parse_bytes(&data)
            .map(ErgoBox)
            .map_err(to_js)
    }

    /// Create ErgoBox from ErgoBoxCandidate by adding transaction id
    /// and index of the box in the transaction
    pub fn from_box_candidate(
        candidate: &ErgoBoxCandidate,
        tx_id: &TxId,
        index: u16,
    ) -> Result<ErgoBox, JsValue> {
        let candidate: chain::ergo_box::ErgoBoxCandidate = candidate.0.clone();
        chain::ergo_box::ErgoBox::from_box_candidate(&candidate, tx_id.clone().into(), index)
            .map_err(to_js)
            .map(ErgoBox)
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
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct BoxValue(pub(crate) chain::ergo_box::box_value::BoxValue);

#[wasm_bindgen]
impl BoxValue {
    /// Recommended (safe) minimal box value to use in case box size estimation is unavailable.
    /// Allows box size upto 2777 bytes with current min box value per byte of 360 nanoERGs
    #[allow(non_snake_case)]
    pub fn SAFE_USER_MIN() -> BoxValue {
        BoxValue(chain::ergo_box::box_value::BoxValue::SAFE_USER_MIN)
    }

    /// Number of units inside one ERGO (i.e. one ERG using nano ERG representation)
    #[allow(non_snake_case)]
    pub fn UNITS_PER_ERGO() -> I64 {
        (chain::ergo_box::box_value::BoxValue::UNITS_PER_ERGO as i64).into()
    }

    /// Create from i64 with bounds check
    pub fn from_i64(v: &I64) -> Result<BoxValue, JsValue> {
        Ok(BoxValue(
            chain::ergo_box::box_value::BoxValue::try_from(i64::from(v.clone()) as u64)
                .map_err(to_js)?,
        ))
    }

    /// Get value as signed 64-bit long (I64)
    pub fn as_i64(&self) -> I64 {
        self.0.as_i64().into()
    }

    /// big-endian byte array representation
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_u64().to_be_bytes().to_vec()
    }
}

impl From<&BoxValue> for chain::ergo_box::box_value::BoxValue {
    fn from(v: &BoxValue) -> Self {
        v.0
    }
}

/// Pair of <value, tokens> for an box
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoBoxAssetsData(ergo_lib::wallet::box_selector::ErgoBoxAssetsData);

#[wasm_bindgen]
impl ErgoBoxAssetsData {
    /// Create new instance
    #[wasm_bindgen(constructor)]
    pub fn new(value: &BoxValue, tokens: &Tokens) -> Result<ErgoBoxAssetsData, JsValue> {
        Ok(ErgoBoxAssetsData(
            ergo_lib::wallet::box_selector::ErgoBoxAssetsData {
                value: value.clone().into(),
                tokens: BoxTokens::opt_empty_vec(
                    tokens.clone().0.into_iter().map(Into::into).collect(),
                )
                .map_err(to_js)?,
            },
        ))
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

impl From<ErgoBoxAssetsDataList> for Vec<ergo_lib::wallet::box_selector::ErgoBoxAssetsData> {
    fn from(v: ErgoBoxAssetsDataList) -> Self {
        v.0.iter().map(|i| i.0.clone()).collect()
    }
}
impl From<Vec<ergo_lib::wallet::box_selector::ErgoBoxAssetsData>> for ErgoBoxAssetsDataList {
    fn from(v: Vec<ergo_lib::wallet::box_selector::ErgoBoxAssetsData>) -> Self {
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
