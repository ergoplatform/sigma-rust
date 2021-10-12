//! Trait implementations to simplify conversion of `ergo_lib` errors into `JsValue`s

use std::num::ParseIntError;

use base16::DecodeError;
use ergo_lib::ergotree_ir::chain::address::AddressEncoderError;
use ergo_lib::ergotree_ir::chain::address::AddressError;
use ergo_lib::ergotree_ir::chain::digest32::Digest32Error;
use ergo_lib::ergotree_ir::chain::ergo_box::box_value::BoxValueError;
use ergo_lib::ergotree_ir::chain::token::TokenAmountError;
use ergo_lib::wallet::signing::TxSigningError;
use ergo_lib::{
    chain::ergo_box::box_builder::ErgoBoxCandidateBuilderError,
    ergotree_ir::{
        ergo_tree::{ErgoTreeConstantError, ErgoTreeConstantsParsingError, ErgoTreeError},
        mir::constant::TryExtractFromError,
        serialization::{SigmaParsingError, SigmaSerializationError},
    },
    wallet::{box_selector::BoxSelectorError, tx_builder::TxBuilderError, WalletError},
};
use serde_json::error::Error;
use wasm_bindgen::JsValue;

/// Ideally we'd like to implement `From<E> for JsValue` for a range of different `ergo-lib` error
/// types `E`, but Rust orphan rules prevent this. A way to get around this limitation is to wrap
/// `Jsvalue` within a local type.
pub struct JsValueWrap(JsValue);

/// Converts any error satisfying `Into<JsValueWrap>` into `JsValue`.
pub fn to_js<S: Into<JsValueWrap>>(s: S) -> JsValue {
    s.into().0
}

macro_rules! from_error_to_wrap {
    ($t:ident) => {
        impl std::convert::From<$t> for JsValueWrap {
            fn from(e: $t) -> Self {
                JsValueWrap(JsValue::from_str(&format!("{}", e)))
            }
        }
    };
}

from_error_to_wrap!(AddressError);
from_error_to_wrap!(SigmaParsingError);
from_error_to_wrap!(AddressEncoderError);
from_error_to_wrap!(ErgoBoxCandidateBuilderError);
from_error_to_wrap!(TryExtractFromError);
from_error_to_wrap!(BoxSelectorError);
from_error_to_wrap!(Digest32Error);
from_error_to_wrap!(SigmaSerializationError);
from_error_to_wrap!(Error);
from_error_to_wrap!(BoxValueError);
from_error_to_wrap!(TokenAmountError);
from_error_to_wrap!(TxBuilderError);
from_error_to_wrap!(TxSigningError);
from_error_to_wrap!(WalletError);
from_error_to_wrap!(DecodeError);

macro_rules! from_error_to_wrap_via_debug {
    ($t:ident) => {
        impl std::convert::From<$t> for JsValueWrap {
            fn from(e: $t) -> Self {
                JsValueWrap(JsValue::from_str(&format!("{:?}", e)))
            }
        }
    };
}

from_error_to_wrap_via_debug!(ErgoTreeError);
from_error_to_wrap_via_debug!(ErgoTreeConstantError);
from_error_to_wrap_via_debug!(ErgoTreeConstantsParsingError);
from_error_to_wrap_via_debug!(ParseIntError);
