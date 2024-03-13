//! Ergo transaction

mod data_input;
pub mod ergo_transaction;
pub mod input;
pub mod reduced;
pub(crate) mod storage_rent;
pub mod unsigned;

use bounded_vec::BoundedVec;
use ergo_chain_types::blake2b256_hash;
pub use ergotree_interpreter::eval::context::TxIoVec;
use ergotree_interpreter::eval::env::Env;
use ergotree_interpreter::eval::extract_sigma_boolean;
use ergotree_interpreter::eval::EvalError;
use ergotree_interpreter::eval::ReductionDiagnosticInfo;
use ergotree_interpreter::sigma_protocol::verifier::verify_signature;
use ergotree_interpreter::sigma_protocol::verifier::TestVerifier;
use ergotree_interpreter::sigma_protocol::verifier::VerificationResult;
use ergotree_interpreter::sigma_protocol::verifier::Verifier;
use ergotree_interpreter::sigma_protocol::verifier::VerifierError;
use ergotree_ir::chain::ergo_box::BoxId;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::token::TokenId;
pub use ergotree_ir::chain::tx_id::TxId;
use ergotree_ir::ergo_tree::ErgoTreeError;
use thiserror::Error;

pub use data_input::*;
use ergotree_interpreter::sigma_protocol::prover::ProofBytes;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::serialization::SigmaSerializeResult;
pub use input::*;

use crate::wallet::signing::make_context;
use crate::wallet::signing::TransactionContext;
use crate::wallet::tx_context::TransactionContextError;

use self::storage_rent::try_spend_storage_rent;
use self::unsigned::UnsignedTransaction;

use indexmap::IndexSet;

use std::convert::TryFrom;
use std::convert::TryInto;
use std::iter::FromIterator;
use std::rc::Rc;

use super::ergo_state_context::ErgoStateContext;

/**
 * ErgoTransaction is an atomic state transition operation. It destroys Boxes from the state
 * and creates new ones. If transaction is spending boxes protected by some non-trivial scripts,
 * its inputs should also contain proof of spending correctness - context extension (user-defined
 * key-value map) and data inputs (links to existing boxes in the state) that may be used during
 * script reduction to crypto, signatures that satisfies the remaining cryptographic protection
 * of the script.
 * Transactions are not encrypted, so it is possible to browse and view every transaction ever
 * collected into a block.
 */
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        try_from = "super::json::transaction::TransactionJson",
        into = "super::json::transaction::TransactionJson"
    )
)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Transaction {
    /// transaction id
    pub(crate) tx_id: TxId,
    /// inputs, that will be spent by this transaction.
    pub inputs: TxIoVec<Input>,
    /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
    /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
    /// included in transaction cost and they do not contain spending proofs.
    pub data_inputs: Option<TxIoVec<DataInput>>,

    /// box candidates to be created by this transaction. Differ from [`Self::outputs`] in that
    /// they do not include transaction id and index
    pub output_candidates: TxIoVec<ErgoBoxCandidate>,

    /// Boxes to be created by this transaction. Differ from [`Self::output_candidates`] in that
    /// they include transaction id and index
    pub outputs: TxIoVec<ErgoBox>,
}

impl Transaction {
    /// Maximum number of outputs
    pub const MAX_OUTPUTS_COUNT: usize = u16::MAX as usize;

    /// Creates new transaction from vectors
    pub fn new_from_vec(
        inputs: Vec<Input>,
        data_inputs: Vec<DataInput>,
        output_candidates: Vec<ErgoBoxCandidate>,
    ) -> Result<Transaction, TransactionError> {
        Ok(Transaction::new(
            inputs
                .try_into()
                .map_err(TransactionError::InvalidInputsCount)?,
            BoundedVec::opt_empty_vec(data_inputs)
                .map_err(TransactionError::InvalidDataInputsCount)?,
            output_candidates
                .try_into()
                .map_err(TransactionError::InvalidOutputCandidatesCount)?,
        )?)
    }

    /// Creates new transaction
    pub fn new(
        inputs: TxIoVec<Input>,
        data_inputs: Option<TxIoVec<DataInput>>,
        output_candidates: TxIoVec<ErgoBoxCandidate>,
    ) -> Result<Transaction, SigmaSerializationError> {
        let outputs_with_zero_tx_id =
            output_candidates
                .clone()
                .enumerated()
                .try_mapped_ref(|(idx, bc)| {
                    ErgoBox::from_box_candidate(bc, TxId::zero(), *idx as u16)
                })?;
        let tx_to_sign = Transaction {
            tx_id: TxId::zero(),
            inputs,
            data_inputs,
            output_candidates: output_candidates.clone(),
            outputs: outputs_with_zero_tx_id,
        };
        let tx_id = tx_to_sign.calc_tx_id()?;
        let outputs = output_candidates
            .enumerated()
            .try_mapped_ref(|(idx, bc)| ErgoBox::from_box_candidate(bc, tx_id, *idx as u16))?;
        Ok(Transaction {
            tx_id,
            outputs,
            ..tx_to_sign
        })
    }

    /// Create Transaction from UnsignedTransaction and an array of proofs in the same order as
    /// UnsignedTransaction.inputs
    pub fn from_unsigned_tx(
        unsigned_tx: UnsignedTransaction,
        proofs: Vec<ProofBytes>,
    ) -> Result<Self, TransactionError> {
        let inputs = unsigned_tx
            .inputs
            .enumerated()
            .try_mapped(|(index, unsigned_input)| {
                proofs
                    .get(index)
                    .map(|proof| Input::from_unsigned_input(unsigned_input, proof.clone()))
                    .ok_or_else(|| {
                        TransactionError::InvalidArgument(format!(
                            "no proof for input index: {}",
                            index
                        ))
                    })
            })?;
        Ok(Transaction::new(
            inputs,
            unsigned_tx.data_inputs,
            unsigned_tx.output_candidates,
        )?)
    }

    fn calc_tx_id(&self) -> Result<TxId, SigmaSerializationError> {
        let bytes = self.bytes_to_sign()?;
        Ok(TxId(blake2b256_hash(&bytes)))
    }

    /// Serialized tx with empty proofs
    pub fn bytes_to_sign(&self) -> Result<Vec<u8>, SigmaSerializationError> {
        let empty_proof_inputs = self.inputs.mapped_ref(|i| i.input_to_sign());
        let tx_to_sign = Transaction {
            inputs: empty_proof_inputs,
            ..(*self).clone()
        };
        tx_to_sign.sigma_serialize_bytes()
    }

    /// Get transaction id
    pub fn id(&self) -> TxId {
        self.tx_id
    }

    /// Check the signature of the transaction's input corresponding
    /// to the given input box, guarded by P2PK script
    pub fn verify_p2pk_input(
        &self,
        input_box: ErgoBox,
    ) -> Result<bool, TransactionSignatureVerificationError> {
        #[allow(clippy::unwrap_used)]
        // since we have a tx with tx_id at this point, serialization is safe to unwrap
        let message = self.bytes_to_sign().unwrap();
        let input = self
            .inputs
            .iter()
            .find(|input| input.box_id == input_box.box_id())
            .ok_or_else(|| {
                TransactionSignatureVerificationError::InputNotFound(input_box.box_id())
            })?;
        let sb = extract_sigma_boolean(&input_box.ergo_tree.proposition()?)?;
        Ok(verify_signature(
            sb,
            message.as_slice(),
            input.spending_proof.proof.as_ref(),
        )?)
    }
}

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum TransactionSignatureVerificationError {
    #[error("Input with id {0:?} not found")]
    InputNotFound(BoxId),
    #[error("input signature verification failed: {0:?}")]
    VerifierError(#[from] VerifierError),
    #[error("ErgoTreeError: {0}")]
    ErgoTreeError(#[from] ErgoTreeError),
    #[error("EvalError: {0}")]
    EvalError(#[from] EvalError),
}

/// Returns distinct token ids from all given ErgoBoxCandidate's
pub fn distinct_token_ids<I>(output_candidates: I) -> IndexSet<TokenId>
where
    I: IntoIterator<Item = ErgoBoxCandidate>,
{
    let token_ids: Vec<TokenId> = output_candidates
        .into_iter()
        .flat_map(|b| {
            b.tokens
                .into_iter()
                .flatten()
                .map(|t| t.token_id)
                .collect::<Vec<TokenId>>()
        })
        .collect();
    IndexSet::<_>::from_iter(token_ids)
}

impl SigmaSerializable for Transaction {
    #[allow(clippy::unwrap_used)]
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoLikeTransaction.scala#L112-L112
        w.put_usize_as_u16_unwrapped(self.inputs.len())?;
        self.inputs.iter().try_for_each(|i| i.sigma_serialize(w))?;
        if let Some(data_inputs) = &self.data_inputs {
            w.put_usize_as_u16_unwrapped(data_inputs.len())?;
            data_inputs.iter().try_for_each(|i| i.sigma_serialize(w))?;
        } else {
            w.put_u16(0)?;
        }

        // Serialize distinct ids of tokens in transaction outputs.
        let distinct_token_ids = distinct_token_ids(self.output_candidates.clone());

        // Note that `self.output_candidates` is of type `TxIoVec` which has a max length of
        // `u16::MAX`. Therefore the following unwrap is safe.
        w.put_u32(u32::try_from(distinct_token_ids.len()).unwrap())?;
        distinct_token_ids
            .iter()
            .try_for_each(|t_id| t_id.sigma_serialize(w))?;

        // serialize outputs
        w.put_usize_as_u16_unwrapped(self.output_candidates.len())?;
        self.output_candidates.iter().try_for_each(|o| {
            ErgoBoxCandidate::serialize_body_with_indexed_digests(o, Some(&distinct_token_ids), w)
        })?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoLikeTransaction.scala#L146-L146

        // parse transaction inputs
        let inputs_count = r.get_u16()?;
        let mut inputs = Vec::with_capacity(inputs_count as usize);
        for _ in 0..inputs_count {
            inputs.push(Input::sigma_parse(r)?);
        }

        // parse transaction data inputs
        let data_inputs_count = r.get_u16()?;
        let mut data_inputs = Vec::with_capacity(data_inputs_count as usize);
        for _ in 0..data_inputs_count {
            data_inputs.push(DataInput::sigma_parse(r)?);
        }

        // parse distinct ids of tokens in transaction outputs
        let tokens_count = r.get_u32()?;
        if tokens_count as usize > Transaction::MAX_OUTPUTS_COUNT * ErgoBox::MAX_TOKENS_COUNT {
            return Err(SigmaParsingError::ValueOutOfBounds(
                "too many tokens in transaction".to_string(),
            ));
        }
        let mut token_ids = IndexSet::with_capacity(tokens_count as usize);
        for _ in 0..tokens_count {
            token_ids.insert(TokenId::sigma_parse(r)?);
        }

        // parse outputs
        let outputs_count = r.get_u16()?;
        let mut outputs = Vec::with_capacity(outputs_count as usize);
        for _ in 0..outputs_count {
            outputs.push(ErgoBoxCandidate::parse_body_with_indexed_digests(
                Some(&token_ids),
                r,
            )?)
        }

        Transaction::new_from_vec(inputs, data_inputs, outputs)
            .map_err(|e| SigmaParsingError::Misc(format!("{}", e)))
    }
}

/// Error when working with Transaction
#[allow(missing_docs)]
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum TransactionError {
    #[error("Tx serialization error: {0}")]
    SigmaSerializationError(#[from] SigmaSerializationError),
    #[error("Tx innvalid argument: {0}")]
    InvalidArgument(String),
    #[error("Invalid Tx inputs: {0:?}")]
    InvalidInputsCount(bounded_vec::BoundedVecOutOfBounds),
    #[error("Invalid Tx output_candidates: {0:?}")]
    InvalidOutputCandidatesCount(bounded_vec::BoundedVecOutOfBounds),
    #[error("Invalid Tx data inputs: {0:?}")]
    InvalidDataInputsCount(bounded_vec::BoundedVecOutOfBounds),
    #[error("input with index {0} not found")]
    InputNofFound(usize),
}

/// Errors on transaction verification
#[derive(Error, Debug)]
pub enum TxVerifyError {
    /// TransactionContextError
    #[error("TransactionContextError: {0}")]
    TransactionContextError(#[from] TransactionContextError),
    /// SerializationError
    #[error("Transaction serialization failed: {0}")]
    SerializationError(#[from] SigmaSerializationError),
    /// VerifierError
    #[error("VerifierError: {0}")]
    VerifierError(#[from] VerifierError),
}

/// Verify transaction input's proof
pub fn verify_tx_input_proof(
    tx_context: &TransactionContext<Transaction>,
    state_context: &ErgoStateContext,
    input_idx: usize,
) -> Result<VerificationResult, TxVerifyError> {
    let input = tx_context
        .spending_tx
        .inputs
        .get(input_idx)
        .ok_or(TransactionContextError::InputBoxNotFound(input_idx))?;
    let input_box = tx_context
        .get_input_box(&input.box_id)
        .ok_or(TransactionContextError::InputBoxNotFound(input_idx))?;
    let ctx = Rc::new(make_context(state_context, tx_context, input_idx)?);
    let verifier = TestVerifier;
    let message_to_sign = tx_context.spending_tx.bytes_to_sign()?;
    // Try spending in storage rent, if any condition is not satisfied fallback to normal script validation
    match try_spend_storage_rent(&input, state_context, &ctx) {
        Some(()) => Ok(VerificationResult {
            result: true,
            cost: 0,
            diag: ReductionDiagnosticInfo {
                env: Env::empty(),
                pretty_printed_expr: None,
            },
        }),
        None => Ok(verifier.verify(
            &input_box.ergo_tree,
            &Env::empty(),
            ctx,
            input.spending_proof.proof.clone(),
            message_to_sign.as_slice(),
        )?),
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
pub mod arbitrary {

    use super::*;
    use proptest::prelude::*;
    use proptest::{arbitrary::Arbitrary, collection::vec};

    impl Arbitrary for Transaction {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                vec(any::<Input>(), 1..10),
                vec(any::<DataInput>(), 0..10),
                vec(any::<ErgoBoxCandidate>(), 1..10),
            )
                .prop_map(|(inputs, data_inputs, outputs)| {
                    Self::new_from_vec(inputs, data_inputs, outputs).unwrap()
                })
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
pub mod tests {

    use super::*;

    use ergotree_ir::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn tx_ser_roundtrip(v in any::<Transaction>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }


        #[test]
        fn tx_id_ser_roundtrip(v in any::<TxId>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }

    }

    #[test]
    #[cfg(feature = "json")]
    fn test_tx_id_calc() {
        let json = r#"
        {
      "id": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
      "inputs": [
        {
          "boxId": "9126af0675056b80d1fda7af9bf658464dbfa0b128afca7bf7dae18c27fe8456",
          "spendingProof": {
            "proofBytes": "",
            "extension": {}
          }
        }
      ],
      "dataInputs": [],
      "outputs": [
        {
          "boxId": "b979c439dc698ce5e823b21c722a6e23721af010e4df8c72de0bfd0c3d9ccf6b",
          "value": 74187765000000000,
          "ergoTree": "101004020e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a7017300730110010204020404040004c0fd4f05808c82f5f6030580b8c9e5ae040580f882ad16040204c0944004c0f407040004000580f882ad16d19683030191a38cc7a7019683020193c2b2a57300007473017302830108cdeeac93a38cc7b2a573030001978302019683040193b1a5730493c2a7c2b2a573050093958fa3730673079973089c73097e9a730a9d99a3730b730c0599c1a7c1b2a5730d00938cc7b2a5730e0001a390c1a7730f",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 0
        },
        {
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }
      ]
    }"#;
        let res = serde_json::from_str(json);
        let t: Transaction = res.unwrap();
        let tx_id_str: String = t.id().into();
        assert_eq!(
            "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
            tx_id_str
        )
    }
}
