//! Wallet-related features for Ergo

pub mod box_selector;
pub mod derivation_path;
pub mod ext_pub_key;
pub mod mnemonic;
pub mod multi_sig;
pub mod secret_key;
pub mod signing;
pub mod tx_builder;

use ergotree_interpreter::sigma_protocol::private_input::PrivateInput;
use ergotree_interpreter::sigma_protocol::prover::Prover;
use ergotree_interpreter::sigma_protocol::prover::TestProver;
use secret_key::SecretKey;
use signing::{sign_transaction, TxSigningError};
use thiserror::Error;

use crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::transaction::reduced::ReducedTransaction;
use crate::chain::transaction::Transaction;
use crate::wallet::mnemonic::Mnemonic;
use crate::wallet::multi_sig::TransactionHintsBag;

use self::signing::sign_reduced_transaction;
use self::signing::TransactionContext;

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
    /// Create wallet instance loading secret key from mnemonic
    /// Returns None if a DlogSecretKey cannot be parsed from the provided phrase
    pub fn from_mnemonic(mnemonic_phrase: &str, mnemonic_pass: &str) -> Option<Wallet> {
        let seed = Mnemonic::to_seed(mnemonic_phrase, mnemonic_pass);
        let mut dlog_bytes: [u8; 32] = Default::default();
        dlog_bytes.copy_from_slice(&seed[..32]);
        let secret = SecretKey::dlog_from_bytes(&dlog_bytes)?;

        Some(Wallet::from_secrets(vec![secret]))
    }

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
        tx_context: TransactionContext,
        state_context: &ErgoStateContext,
        tx_hints: &TransactionHintsBag,
    ) -> Result<Transaction, WalletError> {
        sign_transaction(self.prover.as_ref(), tx_context, state_context, tx_hints)
            .map_err(WalletError::from)
    }

    /// Signs a reduced transaction (generating proofs for inputs)
    pub fn sign_reduced_transaction(
        &self,
        reduced_tx: ReducedTransaction,
    ) -> Result<Transaction, WalletError> {
        sign_reduced_transaction(self.prover.as_ref(), reduced_tx).map_err(WalletError::from)
    }
}
