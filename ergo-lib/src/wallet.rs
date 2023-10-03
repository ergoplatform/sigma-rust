//! Wallet-related features for Ergo

pub mod box_selector;
pub mod derivation_path;
pub mod ext_pub_key;
pub mod ext_secret_key;
pub mod miner_fee;
pub mod mnemonic;
#[cfg(feature = "mnemonic_gen")]
pub mod mnemonic_generator;
pub mod multi_sig;
pub mod secret_key;
pub mod signing;
pub mod tx_builder;
pub mod tx_context;

use ergotree_interpreter::sigma_protocol::private_input::PrivateInput;
use ergotree_interpreter::sigma_protocol::prover::Prover;
use ergotree_interpreter::sigma_protocol::prover::ProverError;
use ergotree_interpreter::sigma_protocol::prover::TestProver;
use secret_key::SecretKey;
use signing::{sign_transaction, TxSigningError};
use thiserror::Error;

use crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::transaction::reduced::ReducedTransaction;
use crate::chain::transaction::unsigned::UnsignedTransaction;
use crate::chain::transaction::Input;
use crate::chain::transaction::Transaction;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::wallet::mnemonic::Mnemonic;
use crate::wallet::multi_sig::{
    generate_commitments, generate_commitments_for, TransactionHintsBag,
};

use self::ext_secret_key::ExtSecretKey;
use self::ext_secret_key::ExtSecretKeyError;
use self::signing::sign_message;
use self::signing::sign_reduced_transaction;
use self::signing::sign_tx_input;
use self::signing::TransactionContext;

/// Wallet
pub struct Wallet {
    prover: Box<dyn Prover>,
}

/// Wallet errors
#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Transaction signing error: {0}")]
    TxSigningError(#[from] TxSigningError),

    #[error("Prover error: {0}")]
    ProverError(#[from] ProverError),

    #[error("ExtSecretKeyError: {0}")]
    ExtSecretKeyError(#[from] ExtSecretKeyError),

    #[error("error parsing SecretKey from ExtSecretKey.bytes")]
    SecretKeyParsingError,
}

impl Wallet {
    /// Create wallet instance loading secret key from mnemonic
    /// Returns None if a DlogSecretKey cannot be parsed from the provided phrase
    pub fn from_mnemonic(
        mnemonic_phrase: &str,
        mnemonic_pass: &str,
    ) -> Result<Wallet, WalletError> {
        let seed = Mnemonic::to_seed(mnemonic_phrase, mnemonic_pass);
        let ext_sk = ExtSecretKey::derive_master(seed)?;
        Ok(Wallet::from_secrets(vec![ext_sk.secret_key()]))
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

    /// Add a new secret to the wallet prover
    pub fn add_secret(&mut self, secret: SecretKey) {
        self.prover.append_secret(secret.into())
    }

    /// Signs a transaction
    pub fn sign_transaction(
        &self,
        tx_context: TransactionContext<UnsignedTransaction>,
        state_context: &ErgoStateContext,
        tx_hints: Option<&TransactionHintsBag>,
    ) -> Result<Transaction, WalletError> {
        sign_transaction(self.prover.as_ref(), tx_context, state_context, tx_hints)
            .map_err(WalletError::from)
    }

    /// Signs a reduced transaction (generating proofs for inputs)
    pub fn sign_reduced_transaction(
        &self,
        reduced_tx: ReducedTransaction,
        tx_hints: Option<&TransactionHintsBag>,
    ) -> Result<Transaction, WalletError> {
        sign_reduced_transaction(self.prover.as_ref(), reduced_tx, tx_hints)
            .map_err(WalletError::from)
    }

    /// Generate commitments for Transaction by wallet secrets
    pub fn generate_commitments(
        &self,
        tx_context: TransactionContext<UnsignedTransaction>,
        state_context: &ErgoStateContext,
    ) -> Result<TransactionHintsBag, TxSigningError> {
        let public_keys: Vec<SigmaBoolean> = self
            .prover
            .secrets()
            .iter()
            .map(|secret| secret.public_image())
            .collect();
        generate_commitments(tx_context, state_context, public_keys.as_slice())
    }

    /// Generate Commitments for reduced Transaction
    pub fn generate_commitments_for_reduced_transaction(
        &self,
        reduced_tx: ReducedTransaction,
    ) -> Result<TransactionHintsBag, TxSigningError> {
        let mut tx_hints = TransactionHintsBag::empty();
        let public_keys: Vec<SigmaBoolean> = self
            .prover
            .secrets()
            .iter()
            .map(|secret| secret.public_image())
            .collect();
        for (index, input) in reduced_tx.reduced_inputs().iter().enumerate() {
            let sigma_prop = input.clone().sigma_prop;
            let hints = generate_commitments_for(&sigma_prop, &public_keys);
            tx_hints.add_hints_for_input(index, hints);
        }
        Ok(tx_hints)
    }

    /// Signs a message
    pub fn sign_message(
        &self,
        sigma_tree: SigmaBoolean,
        msg: &[u8],
    ) -> Result<Vec<u8>, WalletError> {
        sign_message(self.prover.as_ref(), sigma_tree, msg).map_err(WalletError::from)
    }

    /// Signs a transaction input
    pub fn sign_tx_input(
        &self,
        input_idx: usize,
        tx_context: TransactionContext<UnsignedTransaction>,
        state_context: &ErgoStateContext,
        tx_hints: Option<&TransactionHintsBag>,
    ) -> Result<Input, WalletError> {
        let tx = tx_context.spending_tx.clone();
        let message_to_sign = tx.bytes_to_sign().map_err(TxSigningError::from)?;
        Ok(sign_tx_input(
            self.prover.as_ref(),
            &tx_context,
            state_context,
            tx_hints,
            input_idx,
            message_to_sign.as_slice(),
        )?)
    }
}
