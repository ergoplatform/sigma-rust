//! Wallet-related features for Ergo

pub mod box_selector;
pub mod derivation_path;
mod deterministic;
pub mod ext_pub_key;
pub mod ext_secret_key;
pub mod miner_fee;
pub mod mnemonic;
#[cfg(feature = "mnemonic_gen")]
pub mod mnemonic_generator;
#[cfg(feature = "std")]
pub mod multi_sig;
pub mod secret_key;
pub mod signing;
pub mod tx_builder;
pub mod tx_context;

use crate::ergotree_interpreter::sigma_protocol::prover::hint::{Hint, HintsBag};
use alloc::boxed::Box;
use alloc::vec::Vec;
use ergotree_interpreter::sigma_protocol::private_input::PrivateInput;
use ergotree_interpreter::sigma_protocol::prover::Prover;
use ergotree_interpreter::sigma_protocol::prover::ProverError;
use ergotree_interpreter::sigma_protocol::prover::TestProver;
use hashbrown::HashMap;
use secret_key::SecretKey;
use thiserror::Error;

use crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::transaction::reduced::reduce_tx;
use crate::chain::transaction::reduced::ReducedTransaction;
use crate::chain::transaction::unsigned::UnsignedTransaction;
#[cfg(feature = "std")]
use crate::chain::transaction::Input;
use crate::chain::transaction::Transaction;
#[cfg(feature = "std")]
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::wallet::mnemonic::Mnemonic;
#[cfg(feature = "std")]
use crate::wallet::multi_sig::{generate_commitments, generate_commitments_for};

use self::ext_secret_key::ExtSecretKey;
use self::ext_secret_key::ExtSecretKeyError;
use self::signing::sign_reduced_transaction;
use self::signing::TransactionContext;
use self::signing::TxSigningError;
#[cfg(feature = "std")]
use self::signing::{make_context, sign_message, sign_transaction, sign_tx_input};

/// Wallet
pub struct Wallet {
    prover: Box<dyn Prover + Send + Sync>,
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
    #[cfg(feature = "std")]
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
    #[cfg(feature = "std")]
    pub fn sign_reduced_transaction(
        &self,
        reduced_tx: ReducedTransaction,
        tx_hints: Option<&TransactionHintsBag>,
    ) -> Result<Transaction, WalletError> {
        sign_reduced_transaction(self.prover.as_ref(), reduced_tx, tx_hints)
            .map_err(WalletError::from)
    }

    /// Generate commitments for Transaction by wallet secrets
    #[cfg(feature = "std")]
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
    #[cfg(feature = "std")]
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

    /// Generate commitments for P2PK inputs using deterministic nonces. \
    /// Rejects unsupported reductions and inputs without a matching secret key.
    /// Trivial true inputs need no commitments; trivial false inputs are rejected.
    /// See: [`Wallet::sign_transaction_deterministic`]
    pub fn generate_deterministic_commitments(
        &self,
        reduced_tx: &ReducedTransaction,
        aux_rand: &[u8],
    ) -> Result<TransactionHintsBag, TxSigningError> {
        reduced_tx.validate()?;
        let mut tx_hints = TransactionHintsBag::empty();
        let msg = reduced_tx.unsigned_tx.bytes_to_sign()?;
        for (index, input) in reduced_tx.reduced_inputs().iter().enumerate() {
            let bag = self::deterministic::generate_commitments_for(
                &*self.prover,
                &input.sigma_prop,
                &msg,
                aux_rand,
            )
            .map_err(|error| TxSigningError::ProverError(error, index))?;
            tx_hints.add_hints_for_input(index, bag);
        }
        Ok(tx_hints)
    }

    /// Generate signatures for P2PK inputs deterministically
    ///
    /// Schnorr signatures need an unpredictable nonce added to the signature to avoid private key leakage. Normally this is generated using 32 bytes of entropy, but on platforms where that
    /// is not available, `sign_transaction_deterministic` can be used to generate the nonce using a hash of the private key and message. \
    /// Additionally `aux_rand` can be optionally supplied with up 32 bytes of entropy.
    /// # Limitations
    /// Inputs must reduce to a single public key with a matching wallet secret, or to trivial true.
    /// Other reductions, including proveDhTuple, n-of-n, t-of-n and trivial false, are rejected before proving.
    pub fn sign_transaction_deterministic(
        &self,
        tx_context: TransactionContext<UnsignedTransaction>,
        state_context: &ErgoStateContext,
        aux_rand: &[u8],
    ) -> Result<Transaction, WalletError> {
        let reduced_tx = reduce_tx(tx_context, state_context)?;
        let hints = self.generate_deterministic_commitments(&reduced_tx, aux_rand)?;
        sign_reduced_transaction(&*self.prover, reduced_tx, Some(&hints)).map_err(From::from)
    }

    /// Generate signatures for P2PK inputs deterministically
    /// See: [`Wallet::sign_transaction_deterministic`]
    pub fn sign_reduced_transaction_deterministic(
        &self,
        reduced_tx: ReducedTransaction,
        aux_rand: &[u8],
    ) -> Result<Transaction, WalletError> {
        let hints = self.generate_deterministic_commitments(&reduced_tx, aux_rand)?;
        sign_reduced_transaction(&*self.prover, reduced_tx, Some(&hints)).map_err(From::from)
    }

    /// Signs a message
    #[cfg(feature = "std")]
    pub fn sign_message(
        &self,
        sigma_tree: SigmaBoolean,
        msg: &[u8],
    ) -> Result<Vec<u8>, WalletError> {
        sign_message(self.prover.as_ref(), sigma_tree, msg).map_err(WalletError::from)
    }

    /// Signs a transaction input
    #[cfg(feature = "std")]
    pub fn sign_tx_input(
        &self,
        input_idx: usize,
        tx_context: TransactionContext<UnsignedTransaction>,
        state_context: &ErgoStateContext,
        tx_hints: Option<&TransactionHintsBag>,
    ) -> Result<Input, WalletError> {
        let tx = tx_context.spending_tx.clone();
        let message_to_sign = tx.bytes_to_sign().map_err(TxSigningError::from)?;
        let mut context =
            make_context(state_context, &tx_context, input_idx).map_err(TxSigningError::from)?;
        Ok(sign_tx_input(
            self.prover.as_ref(),
            &tx_context,
            state_context,
            &mut context,
            tx_hints,
            input_idx,
            message_to_sign.as_slice(),
        )?)
    }
}

#[cfg(feature = "arbitrary")]
use proptest::prelude::Strategy;
/// TransactionHintsBag
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        try_from = "crate::chain::json::hint::TransactionHintsBagJson",
        into = "crate::chain::json::hint::TransactionHintsBagJson"
    )
)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
#[derive(PartialEq, Debug, Clone)]
pub struct TransactionHintsBag {
    #[cfg_attr(
        feature = "arbitrary",
        proptest(
            strategy = "proptest::collection::hash_map(proptest::prelude::any::<usize>(), proptest::prelude::any::<HintsBag>(), 0..5).prop_map(HashMap::from_iter)"
        )
    )]
    pub(crate) secret_hints: HashMap<usize, HintsBag>,
    #[cfg_attr(
        feature = "arbitrary",
        proptest(
            strategy = "proptest::collection::hash_map(proptest::prelude::any::<usize>(), proptest::prelude::any::<HintsBag>(), 0..5).prop_map(HashMap::from_iter)"
        )
    )]
    pub(crate) public_hints: HashMap<usize, HintsBag>,
}

impl TransactionHintsBag {
    /// Empty TransactionHintsBag
    pub fn empty() -> Self {
        TransactionHintsBag {
            secret_hints: HashMap::new(),
            public_hints: HashMap::new(),
        }
    }

    /// Private helper function to correctly segregate a HintsBag into secret and public parts.
    /// Secret hints are `SecretProven` and `CommitmentHint::OwnCommitment`. All others are public.
    fn segregate_hints(hints_bag: HintsBag) -> (HintsBag, HintsBag) {
        let (secret, public): (Vec<Hint>, Vec<Hint>) =
            hints_bag.hints.into_iter().partition(|hint| {
                matches!(
                    hint,
                    Hint::SecretProven(_) | Hint::CommitmentHint(CommitmentHint::OwnCommitment(_))
                )
            });

        (HintsBag { hints: secret }, HintsBag { hints: public })
    }

    /// Replacing Hints for an input index
    pub fn replace_hints_for_input(&mut self, index: usize, hints_bag: HintsBag) {
        let (secret_bag, public_bag) = Self::segregate_hints(hints_bag);
        self.secret_hints.insert(index, secret_bag);
        self.public_hints.insert(index, public_bag);
    }

    /// Adding hints for a input index
    pub fn add_hints_for_input(&mut self, index: usize, hints_bag: HintsBag) {
        let (mut new_secret_bag, mut new_public_bag) = Self::segregate_hints(hints_bag);

        // Get the existing secret hints, or an empty bag if none exist, and add the new ones.
        if let Some(existing_secrets) = self.secret_hints.get_mut(&index) {
            existing_secrets.hints.append(&mut new_secret_bag.hints);
        } else {
            self.secret_hints.insert(index, new_secret_bag);
        }

        // Get the existing public hints, or an empty bag if none exist, and add the new ones.
        if let Some(existing_public) = self.public_hints.get_mut(&index) {
            existing_public.hints.append(&mut new_public_bag.hints);
        } else {
            self.public_hints.insert(index, new_public_bag);
        }
    }

    /// Outputting HintsBag corresponding for an index
    pub fn all_hints_for_input(&self, index: usize) -> HintsBag {
        let mut all_hints = Vec::new();

        if let Some(secret_bag) = self.secret_hints.get(&index) {
            all_hints.extend_from_slice(&secret_bag.hints);
        }

        if let Some(public_bag) = self.public_hints.get(&index) {
            all_hints.extend_from_slice(&public_bag.hints);
        }

        HintsBag { hints: all_hints }
    }
}
