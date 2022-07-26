//! Transaction signing

use crate::chain::transaction::reduced::ReducedTransaction;
use crate::chain::transaction::{DataInput, Input};
use crate::chain::{
    ergo_state_context::ErgoStateContext,
    transaction::{unsigned::UnsignedTransaction, Transaction},
};
use ergotree_interpreter::sigma_protocol::prover::hint::HintsBag;
use ergotree_interpreter::sigma_protocol::sig_serializer::SigParsingError;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use std::rc::Rc;

use crate::ergotree_ir::chain::ergo_box::BoxId;
use crate::wallet::multi_sig::TransactionHintsBag;
use ergotree_interpreter::eval::context::{Context, TxIoVec};
use ergotree_interpreter::eval::env::Env;
use ergotree_interpreter::sigma_protocol::prover::Prover;
use ergotree_interpreter::sigma_protocol::prover::ProverError;
use ergotree_interpreter::sigma_protocol::prover::ProverResult;
use thiserror::Error;

/// Errors on transaction signing
#[derive(Error, Debug)]
pub enum TxSigningError {
    /// Error on proving an input
    #[error("Prover error (tx input index {1}): {0}")]
    ProverError(ProverError, usize),
    /// Failed to find an input in `boxes_to_spend`
    #[error("Input box not found (index {0})")]
    InputBoxNotFound(usize),
    /// Too many input boxes
    #[error("A maximum of 255 input boxes is allowed (have {0})")]
    TooManyInputBoxes(usize),
    /// `boxes_to_spend` is empty
    #[error("No Input boxes found")]
    NoInputBoxes,
    /// Too many data input boxes
    #[error("A maximum of 255 data input boxes is allowed (have {0})")]
    TooManyDataInputBoxes(usize),
    /// Failed to find a data input in `data_boxes`
    #[error("Data input box not found (index {0})")]
    DataInputBoxNotFound(usize),
    /// Context creation error
    #[error("Context error: {0}")]
    ContextError(String),
    /// Tx serialization failed (id calculation)
    #[error("Transaction serialization failed: {0}")]
    SerializationError(#[from] SigmaSerializationError),
    /// SigParsingError
    #[error("SigParsingError: {0}")]
    SigParsingError(#[from] SigParsingError),
}

pub use super::tx_context::TransactionContext;

/// Exposes common properties for signed and unsigned transactions
pub trait ErgoTransaction {
    /// input boxes ids
    fn inputs_ids(&self) -> TxIoVec<BoxId>;
    /// data input boxes
    fn data_inputs(&self) -> Option<TxIoVec<DataInput>>;
}

impl ErgoTransaction for UnsignedTransaction {
    fn inputs_ids(&self) -> TxIoVec<BoxId> {
        self.inputs.clone().mapped(|input| input.box_id)
    }

    fn data_inputs(&self) -> Option<TxIoVec<DataInput>> {
        self.data_inputs.clone()
    }
}

impl ErgoTransaction for Transaction {
    fn inputs_ids(&self) -> TxIoVec<BoxId> {
        self.inputs.clone().mapped(|input| input.box_id)
    }

    fn data_inputs(&self) -> Option<TxIoVec<DataInput>> {
        self.data_inputs.clone()
    }
}

/// `self_index` - index of the SELF box in the tx_ctx.spending_tx.inputs
pub fn make_context(
    state_ctx: &ErgoStateContext,
    tx_ctx: &TransactionContext<UnsignedTransaction>,
    self_index: usize,
) -> Result<Context, TxSigningError> {
    let height = state_ctx.pre_header.height;

    // Find self_box by matching BoxIDs
    let self_box = tx_ctx
        .get_input_box(&tx_ctx.spending_tx.inputs.as_vec()[self_index].box_id)
        .ok_or_else(|| TxSigningError::ContextError("self_index is out of bounds".to_string()))?;

    let outputs = tx_ctx
        .spending_tx
        .output_candidates
        .iter()
        .enumerate()
        .map(|(idx, b)| ErgoBox::from_box_candidate(b, tx_ctx.spending_tx.id(), idx as u16))
        .collect::<Result<Vec<ErgoBox>, SigmaSerializationError>>()?;
    let data_inputs_ir = if let Some(data_inputs) = tx_ctx.spending_tx.data_inputs.as_ref() {
        Some(data_inputs.clone().enumerated().try_mapped(|(idx, di)| {
            tx_ctx
                .data_boxes
                .as_ref()
                .ok_or(TxSigningError::DataInputBoxNotFound(idx))?
                .iter()
                .find(|b| di.box_id == b.box_id())
                .map(|b| Rc::new(b.clone()))
                .ok_or(TxSigningError::DataInputBoxNotFound(idx))
        })?)
    } else {
        None
    };
    let self_box_ir = Rc::new(self_box);
    let outputs_ir = outputs.into_iter().map(Rc::new).collect();
    let inputs_ir = tx_ctx
        .spending_tx
        .inputs_ids()
        .enumerated()
        .try_mapped(|(idx, u)| {
            tx_ctx
                .get_input_box(&u)
                .map(Rc::new)
                .ok_or(TxSigningError::InputBoxNotFound(idx))
        })?;
    let extension = tx_ctx
        .spending_tx
        .inputs
        .get(self_index)
        .ok_or_else(|| {
            TxSigningError::ContextError(
                "self_index not found in spending transaction inputs".to_string(),
            )
        })?
        .extension
        .clone();
    Ok(Context {
        height,
        self_box: self_box_ir,
        outputs: outputs_ir,
        data_inputs: data_inputs_ir,
        inputs: inputs_ir,
        pre_header: state_ctx.pre_header.clone(),
        extension,
        headers: state_ctx.headers.clone(),
    })
}

/// Signs a transaction (generating proofs for inputs)
pub fn sign_transaction(
    prover: &dyn Prover,
    tx_context: TransactionContext<UnsignedTransaction>,
    state_context: &ErgoStateContext,
    tx_hints: Option<&TransactionHintsBag>,
) -> Result<Transaction, TxSigningError> {
    let tx = tx_context.spending_tx.clone();
    let message_to_sign = tx.bytes_to_sign()?;
    let signed_inputs = tx.inputs.enumerated().try_mapped(|(idx, input)| {
        let input_box = tx_context
            .get_input_box(&input.box_id)
            .ok_or(TxSigningError::InputBoxNotFound(idx))?;
        let ctx = Rc::new(make_context(state_context, &tx_context, idx)?);
        let mut hints_bag = HintsBag::empty();
        if let Some(bag) = tx_hints {
            hints_bag = bag.all_hints_for_input(idx);
        }
        prover
            .prove(
                &input_box.ergo_tree,
                &Env::empty(),
                ctx,
                message_to_sign.as_slice(),
                &hints_bag,
            )
            .map(|proof| Input::new(input.box_id.clone(), proof.into()))
            .map_err(|e| TxSigningError::ProverError(e, idx))
    })?;
    Ok(Transaction::new(
        signed_inputs,
        tx.data_inputs,
        tx.output_candidates,
    )?)
}

/// Signs a reduced transaction (generating proofs for inputs)
pub fn sign_reduced_transaction(
    prover: &dyn Prover,
    reduced_tx: ReducedTransaction,
    tx_hints: Option<&TransactionHintsBag>,
) -> Result<Transaction, TxSigningError> {
    let tx = reduced_tx.unsigned_tx.clone();
    let message_to_sign = tx.bytes_to_sign()?;
    let signed_inputs = tx.inputs.enumerated().try_mapped(|(idx, input)| {
        let inputs = reduced_tx.reduced_inputs();

        // `idx` is valid since it's indexing over `tx.inputs`
        #[allow(clippy::unwrap_used)]
        let reduced_input = inputs.get(idx).unwrap();
        let mut hints_bag = HintsBag::empty();
        if let Some(bag) = tx_hints {
            hints_bag = bag.all_hints_for_input(idx);
        }
        prover
            .generate_proof(
                reduced_input.reduction_result.sigma_prop.clone(),
                message_to_sign.as_slice(),
                &hints_bag,
            )
            .map(|proof| ProverResult {
                proof,
                extension: reduced_input.extension.clone(),
            })
            .map(|proof| Input::new(input.box_id.clone(), proof.into()))
            .map_err(|e| TxSigningError::ProverError(e, idx))
    })?;
    Ok(Transaction::new(
        signed_inputs,
        tx.data_inputs,
        tx.output_candidates,
    )?)
}

/// Sign arbitrary message under a key representing a statement provable via a sigma-protocol.
/// A statement can be a simple ProveDlog (PK) or a complex sigma conjectives tree
pub fn sign_message(
    prover: &dyn Prover,
    sigma_tree: SigmaBoolean,
    msg: &[u8],
) -> Result<Vec<u8>, ProverError> {
    prover
        .generate_proof(sigma_tree, msg, &HintsBag::empty())
        .map(Vec::from)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
    use ergotree_interpreter::sigma_protocol::private_input::PrivateInput;
    use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
    use ergotree_interpreter::sigma_protocol::prover::TestProver;
    use ergotree_interpreter::sigma_protocol::verifier::verify_signature;
    use ergotree_interpreter::sigma_protocol::verifier::TestVerifier;
    use ergotree_interpreter::sigma_protocol::verifier::Verifier;
    use ergotree_interpreter::sigma_protocol::verifier::VerifierError;
    use ergotree_ir::chain::address::AddressEncoder;
    use ergotree_ir::chain::address::NetworkPrefix;
    use ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
    use ergotree_ir::chain::tx_id::TxId;
    use ergotree_ir::serialization::SigmaSerializable;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use rand::prelude::SliceRandom;
    use rand::thread_rng;
    use sigma_test_util::force_any_val;

    use crate::chain::transaction::reduced::reduce_tx;
    use crate::chain::transaction::DataInput;
    use crate::chain::{
        ergo_box::box_builder::ErgoBoxCandidateBuilder, transaction::UnsignedInput,
    };
    use crate::wallet::secret_key::SecretKey;
    use crate::wallet::Wallet;
    use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::mir::expr::Expr;
    use std::convert::TryFrom;
    use std::convert::TryInto;
    use std::rc::Rc;

    fn verify_tx_proofs(
        tx: &Transaction,
        boxes_to_spend: &[ErgoBox],
    ) -> Result<bool, VerifierError> {
        let verifier = TestVerifier;
        let message = tx.bytes_to_sign().unwrap();
        tx.inputs.iter().try_fold(true, |acc, input| {
            let b = boxes_to_spend
                .iter()
                .find(|b| b.box_id() == input.box_id)
                .unwrap();
            let res = verifier.verify(
                &b.ergo_tree,
                &Env::empty(),
                Rc::new(force_any_val::<Context>()),
                input.spending_proof.proof.clone(),
                &message,
            )?;
            Ok(res.result && acc)
        })
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_tx_signing(secrets in vec(any::<DlogProverInput>(), 3..10)) {
            let mut boxes_to_spend: Vec<ErgoBox> = secrets.iter().map(|secret|{
                let pk = secret.public_image();
                let tree = ErgoTree::try_from(Expr::Const(pk.into())).unwrap();
                ErgoBox::new(BoxValue::SAFE_USER_MIN,
                             tree,
                             None,
                             NonMandatoryRegisters::empty(),
                             0,
                             TxId::zero(),
                             0).unwrap()
            }).collect();
            let prover = Rc::new(TestProver {
                secrets: secrets.clone().into_iter().map(PrivateInput::DlogProverInput).collect(),
            });
            let inputs: Vec<UnsignedInput> = boxes_to_spend.clone().into_iter().map(UnsignedInput::from).collect();
            // boxes_to_spend are in the different order to test inputs <-> boxes_to_spend association in the
            // prover (it should not depend on both of them to be in the same order)
            boxes_to_spend.shuffle(&mut thread_rng());
            let ergo_tree = ErgoTree::try_from(Expr::Const(secrets.get(0).unwrap().public_image().into())).unwrap();
            let candidate = ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, ergo_tree, 0)
                .build().unwrap();
            let output_candidates = vec![candidate];
            let tx = UnsignedTransaction::new_from_vec(inputs, vec![], output_candidates).unwrap();
            let tx_context = TransactionContext::new(tx, boxes_to_spend.clone(), vec![]).unwrap();
            let tx_hint_bag=TransactionHintsBag::empty();
            let res = sign_transaction(prover.as_ref(), tx_context.clone(), &force_any_val::<ErgoStateContext>(), Some(&tx_hint_bag));
            let signed_tx = res.unwrap();
            prop_assert!(verify_tx_proofs(&signed_tx, &boxes_to_spend).unwrap());
            let reduced_tx = reduce_tx(tx_context, &force_any_val::<ErgoStateContext>()).unwrap();
            let signed_reduced_tx = sign_reduced_transaction(prover.as_ref(), reduced_tx,None).unwrap();
            prop_assert!(verify_tx_proofs(&signed_reduced_tx, &boxes_to_spend).unwrap());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_tx_context_input_reorderings(
            inputs in vec((any::<ErgoBox>(), any::<ContextExtension>()), 1..10),
            mut data_input_boxes in vec(any::<ErgoBox>(), 1..10),
            candidate in any::<ErgoBoxCandidate>(),
        ) {
          let num_inputs = inputs.len();
          let ut_inputs: Vec<_> = inputs
              .iter()
              .map(|(b, extension)|
                  UnsignedInput {
                    box_id: b.box_id(),
                    extension: extension.clone(),
                  }
              )
              .collect();
          let ut_inputs = TxIoVec::from_vec(ut_inputs).unwrap();

          let mut boxes_to_spend: Vec<_> = inputs.into_iter().map(|(b,_)| b).collect();

          let data_inputs = Some(
              TxIoVec::from_vec(data_input_boxes
                  .clone())
                  .unwrap()
                  .mapped(|b| DataInput{box_id: b.box_id()})
          );

          let expected_data_input_boxes = data_input_boxes.clone();
          let expected_input_boxes = boxes_to_spend.clone();

          // Reverse boxes for `UnsignedTransaction`
          boxes_to_spend.reverse();
          data_input_boxes.reverse();
          let boxes_to_spend = boxes_to_spend;
          let spending_tx = UnsignedTransaction::new(
              ut_inputs,
              data_inputs,
              TxIoVec::from_vec(vec![candidate]).unwrap(),
          ).unwrap();
          let tx_context = TransactionContext::new(
              spending_tx,
              boxes_to_spend,
              data_input_boxes,
          )
          .unwrap();

          let expected_data_input_boxes = Some(TxIoVec::from_vec(expected_data_input_boxes).unwrap().mapped(Rc::new));
          let expected_input_boxes = TxIoVec::from_vec(expected_input_boxes).unwrap().mapped(Rc::new);
          for i in 0..num_inputs {
              let context = make_context(&force_any_val::<ErgoStateContext>(), &tx_context, i).unwrap();
              assert_eq!(expected_data_input_boxes, context.data_inputs);
              assert_eq!(expected_input_boxes, context.inputs);
              assert_eq!(tx_context.spending_tx.inputs.as_vec()[i].box_id, context.self_box.box_id());
          }
        }
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_prover_verify_signature(secret in any::<DlogProverInput>(), message in vec(any::<u8>(), 100..200)) {
            let sb: SigmaBoolean = secret.public_image().into();
            let prover = TestProver {
                secrets: vec![PrivateInput::DlogProverInput(secret)],
            };

            let signature = sign_message(&prover, sb.clone(), message.as_slice()).unwrap();

            prop_assert_eq!(verify_signature(
                                            sb.clone(),
                                            message.as_slice(),
                                            signature.as_slice()).unwrap(),
                            true);

            // possible to append bytes
            let mut ext_signature = signature;
            ext_signature.push(1u8);
            prop_assert_eq!(verify_signature(
                                            sb.clone(),
                                            message.as_slice(),
                                            ext_signature.as_slice()).unwrap(),
                            true);

            // wrong message
            prop_assert_eq!(verify_signature(
                                            sb,
                                            message.as_slice(),
                                            vec![1u8; 100].as_slice()).unwrap(),
                            false);
        }
    }

    #[test]
    fn test_proof_from_mainnet() {
        use crate::chain::transaction::Transaction;

        let tx_json = r#"
         {
      "id": "0e6acf3f18b95bdc5bb1b060baa1eafe53bd89fb08b0e86d6cc00fbdd9e43189",
      "inputs": [
        {
          "boxId": "f353ae1b2027e40ea318e7a2673ea4bbaa281b7acee518a0994c5cbdefb05f55",
          "spendingProof": {
            "proofBytes":"",
            "extension": {}
          }
        },
        {
          "boxId": "56111b039b86f71004b768d2e8b4579f1d79e28e7a617fd5add57a5239498c26",
          "spendingProof": {
            "proofBytes": "6542a8b8914b103dcbc36d77da3bd58e42ca35755a5190b507764b0bae330b924ce86acfa1b5f9bfc8216c3c4628738e8274d902bea06b48",
            "extension": {}
          }
        }
      ],
      "dataInputs": [
        {
          "boxId": "e26d41ed030a30cd563681e72f0b9c07825ac983f8c253a87a43c1da21958ece"
        }
      ],
      "outputs": [
        {
          "boxId": "55be517150fcb7f0f1661ad3ab30f1ac62084b83ad6aa772579bc06cbb52832e",
          "value": 1000000,
          "ergoTree": "100604000400050004000e20b662db51cf2dc39f110a021c2a31c74f0a1a18ffffbf73e8a051a7b8c0f09ebc0e2079974b2314c531e62776e6bc4babff35b37b178cebf0976fc0f416ff34ddbc4fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7ed938cb2db6308720373030001730493cbc272037305cd7202",
          "assets": [
            {
              "tokenId": "12caaacb51c89646fac9a3786eb98d0113bd57d68223ccc11754a4f67281daed",
              "amount": 1
            }
          ],
          "creationHeight": 299218,
          "additionalRegisters": {
            "R4": "070327e65711a59378c59359c3e1d0f7abe906479eccb76094e50fe79d743ccc15e6",
            "R5": "0e20e26d41ed030a30cd563681e72f0b9c07825ac983f8c253a87a43c1da21958ece",
            "R6": "05feaff5de0f"
          },
          "transactionId": "0e6acf3f18b95bdc5bb1b060baa1eafe53bd89fb08b0e86d6cc00fbdd9e43189",
          "index": 0
        },
        {
          "boxId": "fa4a484c855d32a60987a4ddcf1c506aa6bab1c4cb0293c2d5ff35fcd11f2c7b",
          "value": 1000000,
          "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
          "assets": [],
          "creationHeight": 299218,
          "additionalRegisters": {},
          "transactionId": "0e6acf3f18b95bdc5bb1b060baa1eafe53bd89fb08b0e86d6cc00fbdd9e43189",
          "index": 1
        },
        {
          "boxId": "3dee27d0dfb193fd6a263cf2b5b58cab99cb640d1443cd1ce63d909ad3a54197",
          "value": 44516500000,
          "ergoTree": "0008cd0327e65711a59378c59359c3e1d0f7abe906479eccb76094e50fe79d743ccc15e6",
          "assets": [],
          "creationHeight": 299218,
          "additionalRegisters": {},
          "transactionId": "0e6acf3f18b95bdc5bb1b060baa1eafe53bd89fb08b0e86d6cc00fbdd9e43189",
          "index": 2
        }
      ],
      "size": 673
    }
        "#;

        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let decoded_addr = encoder
            .parse_address_from_str("9gmNsqrqdSppLUBqg2UzREmmivgqh1r3jmNcLAc53hk3YCvAGWE")
            .unwrap();

        let ergo_tree = decoded_addr.script().unwrap();

        // let spending_proof_input1 = Base16DecodedBytes::try_from("6542a8b8914b103dcbc36d77da3bd58e42ca35755a5190b507764b0bae330b924ce86acfa1b5f9bfc8216c3c4628738e8274d902bea06b48".to_string()).unwrap();
        let tx: Transaction = serde_json::from_str(tx_json).unwrap();
        let tx_id_str: String = tx.id().into();
        assert_eq!(
            "0e6acf3f18b95bdc5bb1b060baa1eafe53bd89fb08b0e86d6cc00fbdd9e43189",
            tx_id_str
        );
        let message = tx.bytes_to_sign().unwrap();
        let verifier = TestVerifier;
        let ver_res = verifier.verify(
            &ergo_tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            tx.inputs.get(1).unwrap().spending_proof.proof.clone(),
            message.as_slice(),
        );
        assert!(ver_res.unwrap().result);
    }

    #[test]
    fn test_multi_sig_issue_597() {
        let secrets: Vec<SecretKey> = [
            "00eda6c0e9fc808d4cf050fc4e98705372b9f0786a6b63aa4013d1a20539b104",
            "cc2e48e5e53059e0d68866eff97a6037cb39945ea9f09f40fcec82d12cd8cb8b",
            "c97250f41cfa8d545c2f8d75b2ee24002b5feec32340c2bb81fa4e2d4c7527d3",
            "53ceef0ece83401cf5cd853fd0c1a9bbfab750d76f278b3187f1a14768d6e9c4",
        ]
        .iter()
        .map(|s| {
            let sized_bytes: &[u8; DlogProverInput::SIZE_BYTES] =
                &base16::decode(s).unwrap().try_into().unwrap();
            SecretKey::dlog_from_bytes(sized_bytes).unwrap()
        })
        .collect();
        let reduced = ReducedTransaction::sigma_parse_bytes(&base16::decode("ce04022f4cd0df4db787875b3a071e098b72ba4923bd2460e08184b34359563febe04700005e8269c8e2b975a43dc6e74a9c5b10b273313c6d32c1dd40c171fc0a8852ca0100000001a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530480ade204100504000400040004000402d804d601b2a5730000d602e4c6a7041ad603e4c6a70510d604ad7202d901040ecdee7204ea02d19683020193c27201c2a7938cb2db63087201730100018cb2db6308a773020001eb02ea02d19683020193e4c67201041a720293e4c672010510720398b27203730300720498b272037304007204d18b0f010001021a04210302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189210399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f8521024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b6621027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bd100206088094ebdc030008cd0314368e16c9c99c5a6e20dda917aeb826b3a908becff543b3a36b38e6b3355ff5d18b0f0000c0843d1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304d18b0f0000c0af87c3210008cd0314368e16c9c99c5a6e20dda917aeb826b3a908becff543b3a36b38e6b3355ff5d18b0f00009702980304cd0302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189cd0399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f85cd024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b66cd027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bd9604cd0302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189cd0399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f85cd024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b66cd027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bdf39b03d3cb9e02d073").unwrap()).unwrap();
        let prover = Wallet::from_secrets(secrets);
        assert!(prover.sign_reduced_transaction(reduced, None).is_ok());
    }
}
