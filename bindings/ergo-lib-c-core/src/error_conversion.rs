//! Code to simplify conversion of `ergo_lib` errors into local `Error`.

use std::array::TryFromSliceError;
use std::num::ParseIntError;

use base16::DecodeError;
use bounded_vec::BoundedVecOutOfBounds;
use ergo_lib::chain::transaction::TransactionError;
use ergo_lib::ergoscript_compiler::compiler::CompileError;
use ergo_lib::ergotree_ir::chain::address::AddressEncoderError;
use ergo_lib::ergotree_ir::chain::address::AddressError;
use ergo_lib::ergotree_ir::chain::digest32::Digest32Error;
use ergo_lib::ergotree_ir::chain::ergo_box::box_value::BoxValueError;
use ergo_lib::ergotree_ir::chain::token::TokenAmountError;
use ergo_lib::wallet::derivation_path::ChildIndexError;
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
use serde_json::error::Error as SerdeError;

macro_rules! convert_error {
    ($t:ident) => {
        impl std::convert::From<$t> for crate::Error {
            fn from(e: $t) -> Self {
                crate::Error::Misc(format!("{}", e).into())
            }
        }
    };
}

convert_error!(AddressError);
convert_error!(SigmaParsingError);
convert_error!(AddressEncoderError);
convert_error!(ErgoBoxCandidateBuilderError);
convert_error!(TryExtractFromError);
convert_error!(BoxSelectorError);
convert_error!(Digest32Error);
convert_error!(SigmaSerializationError);
convert_error!(SerdeError);
convert_error!(BoxValueError);
convert_error!(TokenAmountError);
convert_error!(TxBuilderError);
convert_error!(TxSigningError);
convert_error!(WalletError);
convert_error!(DecodeError);
convert_error!(TryFromSliceError);

macro_rules! convert_error_via_debug {
    ($t:ident) => {
        impl std::convert::From<$t> for crate::Error {
            fn from(e: $t) -> Self {
                crate::Error::Misc(format!("{:?}", e).into())
            }
        }
    };
}

convert_error_via_debug!(CompileError);
convert_error_via_debug!(ErgoTreeError);
convert_error_via_debug!(ErgoTreeConstantError);
convert_error_via_debug!(ErgoTreeConstantsParsingError);
convert_error_via_debug!(ParseIntError);
convert_error_via_debug!(ChildIndexError);
convert_error_via_debug!(BoundedVecOutOfBounds);
convert_error_via_debug!(TransactionError);
