//! Wallet-related features for Ergo

pub mod box_selector;
pub mod secret_key;
pub mod signing;
pub mod tx_builder;

use secret_key::SecretKey;
use signing::{sign_transaction, TxSigningError};
use thiserror::Error;

use crate::{
    chain::{
        ergo_box::ErgoBox, ergo_state_context::ErgoStateContext,
        transaction::unsigned::UnsignedTransaction, transaction::Transaction,
    },
    sigma_protocol::prover::Prover,
    sigma_protocol::prover::TestProver,
    sigma_protocol::PrivateInput,
};

/// Wallet
pub struct Wallet {
    prover: Box<dyn Prover>,
}

/// Wallet errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum WalletError {
    /// Error on tx signing
    #[error("Transaction signing error: {0}")]
    TxSigningError(TxSigningError),
}

impl From<TxSigningError> for WalletError {
    fn from(e: TxSigningError) -> Self {
        WalletError::TxSigningError(e)
    }
}

impl Wallet {
    /// Create Wallet from secrets
    pub fn from_secrets(secrets: Vec<SecretKey>) -> Wallet {
        let prover = TestProver {
            secrets: secrets.into_iter().map(PrivateInput::from).collect(),
        };
        Wallet {
            prover: Box::new(prover),
        }
    }

    /// Signs a transaction
    pub fn sign_transaction(
        &self,
        tx: UnsignedTransaction,
        boxes_to_spend: &[ErgoBox],
        _data_boxes: &[ErgoBox],
        _state_context: &ErgoStateContext,
    ) -> Result<Transaction, WalletError> {
        sign_transaction(
            self.prover.as_ref(),
            tx,
            boxes_to_spend,
            _data_boxes,
            _state_context,
        )
        .map_err(WalletError::from)
    }
}
