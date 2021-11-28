//! Transaction signing

use ergotree_interpreter::sigma_protocol::prover::hint::HintsBag;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::serialization::SigmaSerializationError;
use std::rc::Rc;

use crate::chain::transaction::reduced::ReducedTransaction;
use crate::chain::transaction::Input;
use crate::chain::{
    ergo_state_context::ErgoStateContext,
    transaction::{unsigned::UnsignedTransaction, Transaction},
};

use crate::wallet::multi_sig::TransactionHintsBag;
use ergotree_interpreter::eval::context::{Context, TxIoVec};
use ergotree_interpreter::eval::env::Env;
use ergotree_interpreter::sigma_protocol::prover::Prover;
use ergotree_interpreter::sigma_protocol::prover::ProverError;
use thiserror::Error;

/// Errors on transaction signing
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum TxSigningError {
    /// Error on proving an input
    #[error("Prover error (tx input index {1}): {0}")]
    ProverError(ProverError, usize),
    /// Failed to find an input in `boxes_to_spend`
    #[error("Input box not found (index {0})")]
    InputBoxNotFound(usize),
    /// Failed to find a data input in `data_boxes`
    #[error("Data input box not found (index {0})")]
    DataInputBoxNotFound(usize),
    /// Context creation error
    #[error("Context error: {0}")]
    ContextError(String),
    /// Tx serialization failed (id calculation)
    #[error("Transaction serialization failed: {0}")]
    SerializationError(#[from] SigmaSerializationError),
}

/// Transaction and an additional info required for signing
#[derive(PartialEq, Debug, Clone)]
pub struct TransactionContext {
    /// Unsigned transaction to sign
    pub spending_tx: UnsignedTransaction,
    /// Boxes corresponding to [`UnsignedTransaction::inputs`]
    boxes_to_spend: TxIoVec<ErgoBox>,
    /// Boxes corresponding to [`UnsignedTransaction::data_inputs`]
    data_boxes: Option<TxIoVec<ErgoBox>>,
}

impl TransactionContext {
    /// Return a `TransactionContext` instance if and only if every unsigned transaction input has
    /// an associated box within `boxes_to_spend` and every data input has an associated box within
    /// `data_boxes`.
    pub fn new(
        spending_tx: UnsignedTransaction,
        boxes_to_spend: TxIoVec<ErgoBox>,
        data_boxes: Option<TxIoVec<ErgoBox>>,
    ) -> Result<Self, TxSigningError> {
        for (i, unsigned_input) in spending_tx.inputs.iter().enumerate() {
            if !boxes_to_spend
                .iter()
                .any(|b| unsigned_input.box_id == b.box_id())
            {
                return Err(TxSigningError::InputBoxNotFound(i));
            }
        }
        if let Some(data_inputs) = spending_tx.data_inputs.as_ref() {
            if let Some(data_boxes) = data_boxes.as_ref() {
                for (i, data_input) in data_inputs.iter().enumerate() {
                    if !data_boxes.iter().any(|b| data_input.box_id == b.box_id()) {
                        return Err(TxSigningError::DataInputBoxNotFound(i));
                    }
                }
            } else {
                return Err(TxSigningError::DataInputBoxNotFound(0));
            }
        }
        Ok(TransactionContext {
            spending_tx,
            boxes_to_spend,
            data_boxes,
        })
    }

    /// Get boxes corresponding to [`UnsignedTransaction::inputs`]
    pub fn get_boxes_to_spend(&self) -> impl Iterator<Item = &ErgoBox> {
        self.boxes_to_spend.iter()
    }
}

/// `self_index` - index of the SELF box in the tx_ctx.spending_tx.inputs
pub fn make_context(
    state_ctx: &ErgoStateContext,
    tx_ctx: &TransactionContext,
    self_index: usize,
) -> Result<Context, TxSigningError> {
    let height = state_ctx.pre_header.height;

    // Find self_box by matching BoxIDs
    let self_box = tx_ctx
        .boxes_to_spend
        .iter()
        .find(|b| b.box_id() == tx_ctx.spending_tx.inputs.as_vec()[self_index].box_id)
        .cloned()
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
        .inputs
        .clone()
        .enumerated()
        .try_mapped(|(idx, u)| {
            tx_ctx
                .boxes_to_spend
                .iter()
                .find(|b| u.box_id == b.box_id())
                .map(|b| Rc::new(b.clone()))
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
    tx_context: TransactionContext,
    state_context: &ErgoStateContext,
    tx_hints: &TransactionHintsBag,
) -> Result<Transaction, TxSigningError> {
    let tx = tx_context.spending_tx.clone();
    let message_to_sign = tx.bytes_to_sign()?;
    let signed_inputs = tx.inputs.enumerated().try_mapped(|(idx, input)| {
        let input_box = tx_context
            .boxes_to_spend
            .iter()
            .find(|b| b.box_id() == input.box_id)
            .ok_or(TxSigningError::InputBoxNotFound(idx))?;
        let ctx = Rc::new(make_context(state_context, &tx_context, idx)?);
        let hints_bag = tx_hints.all_hints_for_input(idx);
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
) -> Result<Transaction, TxSigningError> {
    let tx = reduced_tx.unsigned_tx.clone();
    let message_to_sign = tx.bytes_to_sign()?;
    let signed_inputs = tx.inputs.enumerated().try_mapped(|(idx, input)| {
        let inputs = reduced_tx.reduced_inputs();
        let reduced_input = inputs.get(idx).unwrap();
        prover
            .generate_proof(
                reduced_input.reduction_result.sigma_prop.clone(),
                message_to_sign.as_slice(),
                &HintsBag::empty(),
                reduced_input.extension.clone(),
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
    use ergotree_interpreter::sigma_protocol::private_input::PrivateInput;
    use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
    use ergotree_interpreter::sigma_protocol::prover::TestProver;
    use ergotree_interpreter::sigma_protocol::verifier::TestVerifier;
    use ergotree_interpreter::sigma_protocol::verifier::Verifier;
    use ergotree_interpreter::sigma_protocol::verifier::VerifierError;
    use ergotree_ir::chain::address::AddressEncoder;
    use ergotree_ir::chain::address::NetworkPrefix;
    use ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
    use ergotree_ir::chain::tx_id::TxId;
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
            let tx = UnsignedTransaction::new(inputs.try_into().unwrap(),
                None, output_candidates.try_into().unwrap()).unwrap();
            let tx_context = TransactionContext { spending_tx: tx,
                                                  boxes_to_spend: TxIoVec::from_vec(boxes_to_spend.clone()).unwrap(), data_boxes: None };
            let tx_hint_bag=TransactionHintsBag::empty();
            let res = sign_transaction(prover.as_ref(), tx_context.clone(), &force_any_val::<ErgoStateContext>(), &tx_hint_bag);
            let signed_tx = res.unwrap();
            prop_assert!(verify_tx_proofs(&signed_tx, &boxes_to_spend).unwrap());
            let reduced_tx = reduce_tx(tx_context, &force_any_val::<ErgoStateContext>()).unwrap();
            let signed_reduced_tx = sign_reduced_transaction(prover.as_ref(), reduced_tx).unwrap();
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
          let data_input_boxes = Some(TxIoVec::from_vec(data_input_boxes.clone()).unwrap());
          let boxes_to_spend = TxIoVec::from_vec(boxes_to_spend.clone()).unwrap();
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
}
