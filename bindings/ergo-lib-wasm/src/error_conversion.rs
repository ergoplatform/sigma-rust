//! Trait implementations to simplify conversion of `ergo_lib` errors into `JsValue`s

use std::array::TryFromSliceError;
use std::net::AddrParseError;
use std::num::ParseIntError;

use base16::DecodeError;
use bounded_vec::BoundedVecOutOfBounds;
use ergo_lib::chain::transaction::ergo_transaction::TxValidationError;
use ergo_lib::chain::transaction::TransactionSignatureVerificationError;
use ergo_lib::chain::transaction::TxVerifyError;
use ergo_lib::ergo_chain_types::DigestNError;
use ergo_lib::ergo_nipopow::NipopowProofError;
#[cfg(feature = "rest")]
use ergo_lib::ergo_rest::{NodeError, PeerDiscoveryError};
use ergo_lib::ergotree_interpreter::sigma_protocol::verifier::VerifierError;
use ergo_lib::ergotree_ir::chain::address::AddressEncoderError;
use ergo_lib::ergotree_ir::chain::address::AddressError;
use ergo_lib::ergotree_ir::chain::ergo_box::box_value::BoxValueError;
use ergo_lib::ergotree_ir::chain::ergo_box::RegisterValueError;
use ergo_lib::ergotree_ir::chain::token::TokenAmountError;
use ergo_lib::wallet::derivation_path::ChildIndexError;
use ergo_lib::wallet::derivation_path::DerivationPathError;
use ergo_lib::wallet::ext_pub_key::ExtPubKeyError;
use ergo_lib::wallet::ext_secret_key::ExtSecretKeyError;
use ergo_lib::wallet::signing::TxSigningError;
use ergo_lib::wallet::tx_context::TransactionContextError;
use ergo_lib::{
    chain::ergo_box::box_builder::ErgoBoxCandidateBuilderError,
    ergotree_ir::{
        ergo_tree::{ErgoTreeConstantError, ErgoTreeError},
        mir::constant::TryExtractFromError,
        serialization::{SigmaParsingError, SigmaSerializationError},
    },
    wallet::{box_selector::BoxSelectorError, tx_builder::TxBuilderError, WalletError},
};
use serde_json::error::Error;
#[cfg(feature = "rest")]
use url::ParseError;
use wasm_bindgen::JsValue;

use crate::ast::js_conv::ConvError;

/// Ideally we'd like to implement `From<E> for JsValue` for a range of different `ergo-lib` error
/// types `E`, but Rust orphan rules prevent this. A way to get around this limitation is to wrap
/// `Jsvalue` within a local type.
pub struct JsValueWrap(js_sys::Error);

/// Converts any error satisfying `Into<JsValueWrap>` into `JsValue`.
pub fn to_js<S: Into<JsValueWrap>>(s: S) -> JsValue {
    s.into().0.into()
}

macro_rules! from_error_to_wrap {
    ($t:ident) => {
        impl std::convert::From<$t> for JsValueWrap {
            fn from(e: $t) -> Self {
                let js_err = js_sys::Error::new(&format!("{}", e));
                js_err.set_name(stringify!($t));
                JsValueWrap(js_err)
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
from_error_to_wrap!(DigestNError);
from_error_to_wrap!(SigmaSerializationError);
from_error_to_wrap!(Error);
from_error_to_wrap!(BoxValueError);
from_error_to_wrap!(TokenAmountError);
from_error_to_wrap!(TxBuilderError);
from_error_to_wrap!(TxSigningError);
from_error_to_wrap!(WalletError);
from_error_to_wrap!(DecodeError);
from_error_to_wrap!(TryFromSliceError);
from_error_to_wrap!(AddrParseError);
from_error_to_wrap!(TransactionSignatureVerificationError);
from_error_to_wrap!(ErgoTreeError);
from_error_to_wrap!(ChildIndexError);
from_error_to_wrap!(BoundedVecOutOfBounds);
from_error_to_wrap!(ExtSecretKeyError);
from_error_to_wrap!(DerivationPathError);
from_error_to_wrap!(VerifierError);
from_error_to_wrap!(ExtPubKeyError);
from_error_to_wrap!(ParseIntError);
#[cfg(feature = "rest")]
from_error_to_wrap!(NodeError);
#[cfg(feature = "rest")]
from_error_to_wrap!(PeerDiscoveryError);
#[cfg(feature = "rest")]
from_error_to_wrap!(ParseError);
from_error_to_wrap!(ConvError);
from_error_to_wrap!(TransactionContextError);
from_error_to_wrap!(TxVerifyError);
from_error_to_wrap!(TxValidationError);
from_error_to_wrap!(RegisterValueError);
from_error_to_wrap!(String);

macro_rules! from_error_to_wrap_via_debug {
    ($t:ident) => {
        impl std::convert::From<$t> for JsValueWrap {
            fn from(e: $t) -> Self {
                let js_err = js_sys::Error::new(&format!("{:?}", e));
                js_err.set_name(stringify!($t));
                JsValueWrap(js_err)
            }
        }
    };
}

from_error_to_wrap_via_debug!(ErgoTreeConstantError);
from_error_to_wrap_via_debug!(NipopowProofError);
