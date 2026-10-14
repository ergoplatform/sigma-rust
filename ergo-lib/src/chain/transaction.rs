//! Ergo transaction

mod data_input;
pub mod ergo_transaction;
pub mod input;
pub mod reduced;
pub(crate) mod storage_rent;
pub mod unsigned;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use bounded_vec::BoundedVec;
use ergo_chain_types::blake2b256_hash;
use ergotree_interpreter::eval::env::Env;
use ergotree_interpreter::eval::extract_sigma_boolean;
use ergotree_interpreter::eval::EvalError;
use ergotree_interpreter::eval::ReductionDiagnosticInfo;
use ergotree_interpreter::sigma_protocol::verifier::verify_signature;
use ergotree_interpreter::sigma_protocol::verifier::TestVerifier;
use ergotree_interpreter::sigma_protocol::verifier::VerificationResult;
use ergotree_interpreter::sigma_protocol::verifier::Verifier;
use ergotree_interpreter::sigma_protocol::verifier::VerifierError;
use ergotree_ir::chain::context::Context;
pub use ergotree_ir::chain::context::TxIoVec;
use ergotree_ir::chain::ergo_box::BoxId;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::token::TokenId;
pub use ergotree_ir::chain::tx_id::TxId;
use ergotree_ir::chain::IndexSet;
use ergotree_ir::ergo_tree::ErgoTreeError;
use ergotree_ir::ergo_tree::ErgoTreeVersion;
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

use crate::wallet::signing::update_context;
use crate::wallet::signing::TransactionContext;
use crate::wallet::tx_context::TransactionContextError;

use self::ergo_transaction::TxValidationError;
use self::storage_rent::try_spend_storage_rent;
use self::unsigned::UnsignedTransaction;

use core::convert::TryFrom;
use core::convert::TryInto;
use core::iter::FromIterator;

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
pub fn distinct_token_ids<'a, I>(output_candidates: I) -> IndexSet<TokenId>
where
    I: IntoIterator<Item = &'a ErgoBoxCandidate>,
{
    let token_ids = output_candidates
        .into_iter()
        .flat_map(|b| b.tokens.iter().flatten().map(|t| t.token_id));

    IndexSet::<_>::from_iter(token_ids)
}

impl SigmaSerializable for Transaction {
    #[allow(clippy::unwrap_used)]
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        // Set tree version to V0 to match reference impl where global tree version is V0. This prevents including new types added in V6 (UnsignedBigInt, etc) in registers of boxes/contextextensions
        w.with_tree_version(ErgoTreeVersion::V0, |w| {
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
            let distinct_token_ids = distinct_token_ids(&self.output_candidates);

            // Note that `self.output_candidates` is of type `TxIoVec` which has a max length of
            // `u16::MAX`. Therefore the following unwrap is safe.
            w.put_u32(u32::try_from(distinct_token_ids.len()).unwrap())?;
            distinct_token_ids
                .iter()
                .try_for_each(|t_id| t_id.sigma_serialize(w))?;

            // serialize outputs
            w.put_usize_as_u16_unwrapped(self.output_candidates.len())?;
            self.output_candidates.iter().try_for_each(|o| {
                ErgoBoxCandidate::serialize_body_with_indexed_digests(
                    o,
                    Some(&distinct_token_ids),
                    w,
                )
            })?;
            Ok(())
        })
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        // ergo's `ErgoTransactionSerializer.parse` reads each transaction on a new
        // `SigmaByteReader` (ergo v6.0.6 `ErgoTransaction.scala:497-503`), a block's
        // transactions included (`BlockTransactions.scala:187-200`). So a transaction starts
        // at level 0 with empty stores, whatever an earlier one left on the same stream.
        r.with_fresh_parse_state(|r| {
            r.with_tree_version(ErgoTreeVersion::V0, |r| {
                // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoLikeTransaction.scala#L146-L146

                // parse transaction inputs
                let inputs_count = r.get_u16()?;
                let mut inputs = Vec::new();
                for _ in 0..inputs_count {
                    inputs.push(Input::sigma_parse(r)?);
                }

                // parse transaction data inputs
                let data_inputs_count = r.get_u16()?;
                let mut data_inputs = Vec::new();
                for _ in 0..data_inputs_count {
                    data_inputs.push(DataInput::sigma_parse(r)?);
                }

                // parse distinct ids of tokens in transaction outputs
                let tokens_count = r.get_u32()?;
                if tokens_count as usize
                    > Transaction::MAX_OUTPUTS_COUNT * ErgoBox::MAX_TOKENS_COUNT
                {
                    return Err(SigmaParsingError::ValueOutOfBounds(
                        "too many tokens in transaction".to_string(),
                    ));
                }
                let mut token_ids = IndexSet::with_hasher(Default::default());
                for _ in 0..tokens_count {
                    token_ids.insert(TokenId::sigma_parse(r)?);
                }

                // parse outputs
                let outputs_count = r.get_u16()?;
                let mut outputs = Vec::new();
                for _ in 0..outputs_count {
                    outputs.push(ErgoBoxCandidate::parse_body_with_indexed_digests(
                        Some(&token_ids),
                        r,
                    )?)
                }

                Transaction::new_from_vec(inputs, data_inputs, outputs)
                    .map_err(|e| SigmaParsingError::Misc(format!("{}", e)))
            })
        })
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

/// Verify transaction input's proof
pub fn verify_tx_input_proof<'ctx>(
    tx_context: &'ctx TransactionContext<Transaction>,
    ctx: &mut Context<'ctx>,
    state_context: &ErgoStateContext,
    input_idx: usize,
    bytes_to_sign: &[u8],
) -> Result<VerificationResult, TxValidationError> {
    update_context(ctx, tx_context, input_idx)?;
    let input = tx_context
        .spending_tx
        .inputs
        .get(input_idx)
        .ok_or(TransactionContextError::InputBoxNotFound(input_idx))?;
    let input_box = tx_context
        .get_input_box(&input.box_id)
        .ok_or(TransactionContextError::InputBoxNotFound(input_idx))?;
    let verifier = TestVerifier;
    // Try spending in storage rent, if any condition is not satisfied fallback to normal script validation
    match try_spend_storage_rent(input, input_box, state_context, ctx) {
        Some(()) => Ok(VerificationResult {
            result: true,
            cost: 0,
            diag: ReductionDiagnosticInfo {
                env: Env::empty(),
                pretty_printed_expr: None,
            },
        }),
        None => verifier
            .verify(
                &input_box.ergo_tree,
                ctx,
                input.spending_proof.proof.clone(),
                bytes_to_sign,
            )
            .map_err(|e| TxValidationError::VerifierError(input_idx, e)),
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
mod tests {

    use core::str::FromStr;

    use crate::chain::transaction::prover_result::ProverResult;

    use super::*;

    use ergotree_ir::{
        chain::{
            context_extension::ContextExtension,
            ergo_box::{box_value::BoxValue, NonMandatoryRegisterId, NonMandatoryRegisters},
        },
        ergo_tree::ErgoTree,
        mir::{constant::Constant, val_def::ValId},
        serialization::{
            constant_store::ConstantStore, sigma_byte_reader::from_bytes, sigma_serialize_roundtrip,
        },
        types::stype::SType,
        unsignedbigint256::UnsignedBigInt,
    };
    use indexmap::IndexMap;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

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
    fn test_v6_types() {
        // Test that boxes/contextextension can't contain V6 types
        let mut ergo_box = ErgoBoxCandidate {
            value: BoxValue::SAFE_USER_MIN,
            ergo_tree: force_any_val::<ErgoTree>(),
            tokens: None,
            additional_registers: NonMandatoryRegisters::new([(
                NonMandatoryRegisterId::R4,
                Constant::from(UnsignedBigInt::from_str("0").unwrap()),
            )])
            .unwrap(),
            creation_height: 0,
        };
        assert!(matches!(
            Transaction::new_from_vec(
                vec![Input::new(
                    BoxId::zero(),
                    ProverResult {
                        proof: ProofBytes::Empty,
                        extension: ContextExtension::empty(),
                    },
                )],
                vec![],
                vec![ergo_box.clone()],
            ),
            Err(TransactionError::SigmaSerializationError(_))
        ));
        ergo_box.additional_registers = NonMandatoryRegisters::empty();
        assert!(matches!(
            Transaction::new_from_vec(
                vec![Input::new(
                    BoxId::zero(),
                    ProverResult {
                        proof: ProofBytes::Empty,
                        extension: ContextExtension {
                            values: IndexMap::from_iter([(
                                0,
                                UnsignedBigInt::from_str("0").unwrap().into()
                            )])
                        },
                    },
                )],
                vec![],
                vec![ergo_box.clone()],
            ),
            Err(TransactionError::SigmaSerializationError(_))
        ));
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

    /// Transaction::sigma_parse with a huge declared inputs count but no data
    /// must return Err without a multi-gigabyte pre-allocation.
    #[test]
    fn transaction_parse_huge_inputs_count_returns_err() {
        use ergotree_ir::serialization::SigmaSerializable;
        use sigma_ser::vlq_encode::WriteSigmaVlqExt;
        let mut data = Vec::new();
        let mut w =
            ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter::new(&mut data, None);
        w.put_u16(u16::MAX).unwrap();
        let result = Transaction::sigma_parse_bytes(&data);
        assert!(result.is_err());
    }

    /// A transaction spending one input (empty proof, context extension `extension`), with
    /// no data inputs and one 1 ERG output guarded by `tree` (creation height 1, no tokens,
    /// no registers)
    fn tx_bytes(extension: &[u8], tree: &[u8]) -> Vec<u8> {
        let mut bytes = vec![1];
        bytes.extend(
            base16::decode("a05d90c50251aea28100ccaa1da38004fb5a04154bfa300a600f3c10d130816a")
                .unwrap(),
        );
        bytes.push(0); // proof
        bytes.extend_from_slice(extension);
        bytes.extend_from_slice(&[0, 0, 1]); // data inputs, token ids, outputs
        bytes.extend(base16::decode("8094ebdc03").unwrap());
        bytes.extend_from_slice(tree);
        bytes.extend_from_slice(&[1, 0, 0]); // creation height, tokens, registers
        bytes
    }

    /// A size-flagged v0 tree `BoolToSigmaProp(LogicalNot^107(<0x75>))`: the unknown opcode
    /// 0x75 degrades it to `Unparsed`, leaving 109 levels behind on the reader
    fn tx_degrading_at_level_109() -> Vec<u8> {
        let mut tree = vec![0x08, 0x6d, 0xd1];
        tree.extend(core::iter::repeat_n(0xef, 107));
        tree.push(0x75);
        tx_bytes(&[0], &tree)
    }

    /// `BlockValue { ValDef(1, sigmaProp(true)) } ValUse(1)`
    fn tx_defining_val_1() -> Vec<u8> {
        tx_bytes(
            &[0],
            &[0x00, 0xd8, 0x01, 0xd6, 0x01, 0x08, 0xd3, 0x72, 0x01],
        )
    }

    /// Run `f` on a thread with a 32 MiB stack: a debug build takes ~40 KB of stack per
    /// expression level, and `tx_degrading_at_level_109` nests 109 of them.
    fn on_deep_stack(f: impl FnOnce() + Send + 'static) {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(f)
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn transactions_on_one_reader_each_start_at_level_zero() {
        // The second transaction's `sigmaProp(true)` output takes two levels, which a reader
        // still at the first transaction's 109 would refuse at 111.
        on_deep_stack(|| {
            let second = tx_bytes(&[0], &[0x00, 0x08, 0xd3]);
            let mut r = from_bytes([tx_degrading_at_level_109(), second].concat());
            Transaction::sigma_parse(&mut r).unwrap();
            Transaction::sigma_parse(&mut r).unwrap();
        });
    }

    #[test]
    fn transactions_on_one_reader_do_not_share_val_def_types() {
        // `ValUse(1)` without a `ValDef` is rejected (sigmastate's `ValDefTypeStore` lookup
        // throws), also after a transaction that defined 1 on the same stream.
        let undefined_val_use = tx_bytes(&[0], &[0x00, 0x72, 0x01]);
        let mut r = from_bytes([tx_defining_val_1(), undefined_val_use].concat());
        Transaction::sigma_parse(&mut r).unwrap();
        assert!(matches!(
            Transaction::sigma_parse(&mut r),
            Err(SigmaParsingError::ValDefIdNotFound(ValId(1)))
        ));
    }

    #[test]
    fn transaction_parse_leaves_the_reader_state_as_it_found_it() {
        on_deep_stack(|| {
            for tx in [tx_degrading_at_level_109(), tx_defining_val_1()] {
                let mut r = from_bytes(&tx);
                r.set_level(5).unwrap();
                r.set_constant_store(ConstantStore::new(vec![1i32.into()]));
                r.val_def_type_store().insert(ValId(7), SType::SInt);
                r.set_deserialize(true);
                Transaction::sigma_parse(&mut r).unwrap();
                assert_eq!(r.level(), 5);
                assert_eq!(r.constant_store().get(0).unwrap().tpe, SType::SInt);
                assert_eq!(r.val_def_type_store().get(&ValId(7)), Some(&SType::SInt));
                assert!(r.val_def_type_store().get(&ValId(1)).is_none());
                assert!(r.was_deserialize());
            }
        });
    }

    #[test]
    fn function_type_code_in_a_transaction_is_an_error_not_a_panic() {
        // A transaction parses at ErgoTree version 0, where sigmastate's `CheckTypeCode`
        // rejects type code 112 (the function type from v3): in a context extension value
        // and in a register alike.
        let sfunc = [0x70, 0x01, 0x04, 0x04, 0x00]; // (Int) => Int
        let in_extension = tx_bytes(&[&[0x01, 0x01][..], &sfunc].concat(), &[0x00, 0x08, 0xd3]);
        let mut in_register = tx_bytes(&[0], &[0x00, 0x08, 0xd3]);
        in_register.pop(); // registers count
        in_register.push(1);
        in_register.extend_from_slice(&sfunc);
        for tx in [in_extension, in_register] {
            assert!(matches!(
                Transaction::sigma_parse_bytes(&tx),
                Err(SigmaParsingError::InvalidTypeCode(112))
            ));
        }
    }
}
