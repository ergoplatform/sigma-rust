//! Transaction signing

use crate::{
    chain::{
        ergo_box::ErgoBox,
        ergo_state_context::ErgoStateContext,
        input::Input,
        transaction::{unsigned::UnsignedTransaction, Transaction},
    },
    eval::Env,
    sigma_protocol::prover::{Prover, ProverError},
};

/// Errors on transaction signing
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TxSigningError {
    /// error on proving an input
    ProverError(ProverError, usize),
    /// failed to find an input in boxes_to_spend
    InputBoxNotFound(usize),
}

/// Signs a transaction (generating proofs for inputs)
pub fn sign_transaction<T: Prover>(
    prover: T,
    tx: UnsignedTransaction,
    boxes_to_spend: &[ErgoBox],
    _data_boxes: &[ErgoBox],
    _state_context: &ErgoStateContext,
) -> Result<Transaction, TxSigningError> {
    let message_to_sign = tx.bytes_to_sign();
    let mut signed_inputs: Vec<Input> = vec![];
    boxes_to_spend
        .iter()
        .enumerate()
        .try_for_each(|(idx, input_box)| {
            if let Some(unsigned_input) = tx.inputs.get(idx) {
                prover
                    .prove(
                        &input_box.ergo_tree,
                        &Env::empty(),
                        message_to_sign.as_slice(),
                    )
                    .map(|proof| {
                        let input = Input {
                            box_id: unsigned_input.box_id.clone(),
                            spending_proof: proof,
                        };
                        signed_inputs.push(input);
                    })
                    .map_err(|e| TxSigningError::ProverError(e, idx))
            } else {
                Err(TxSigningError::InputBoxNotFound(idx))
            }
        })?;
    Ok(Transaction::new(
        signed_inputs,
        tx.data_inputs,
        tx.output_candidates,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::{
        ast::{Constant, Expr},
        chain::{
            ergo_box::{register::NonMandatoryRegisters, ErgoBoxCandidate},
            input::UnsignedInput,
            transaction::TxId,
        },
        sigma_protocol::{
            prover::TestProver,
            verifier::{TestVerifier, Verifier, VerifierError},
            DlogProverInput, PrivateInput,
        },
        types::SType,
        ErgoTree,
    };
    use std::{convert::TryInto, rc::Rc};

    fn verify_tx_proofs(
        tx: &Transaction,
        boxes_to_spend: &[ErgoBox],
    ) -> Result<bool, VerifierError> {
        let verifier = TestVerifier;
        let message = tx.bytes_to_sign();
        boxes_to_spend
            .iter()
            .zip(tx.inputs.clone())
            .try_fold(true, |acc, (b, input)| {
                let res = verifier.verify(
                    &b.ergo_tree,
                    &Env::empty(),
                    &input.spending_proof.proof,
                    &message,
                )?;
                Ok(res.result && acc)
            })
    }

    proptest! {

        #[test]
        fn test_tx_signing(secrets in vec(any::<DlogProverInput>(), 1..10)) {
            let boxes_to_spend: Vec<ErgoBox> = secrets.iter().map(|secret|{
                let pk = secret.public_image();
                let tree = ErgoTree::from(Rc::new(Expr::Const(Constant {
                    tpe: SType::SSigmaProp,
                    v: pk.into(),
                })));
                ErgoBox::new(1u64.try_into().unwrap(), tree, vec![], NonMandatoryRegisters::empty(), 0, TxId::zero(), 0)
            }).collect();
            let prover = TestProver {
                secrets: secrets.clone().into_iter().map(PrivateInput::DlogProverInput).collect(),
            };
            let inputs = boxes_to_spend.clone().into_iter().map(UnsignedInput::from).collect();
            let ergo_tree = ErgoTree::from(Rc::new(Expr::Const(Constant {
                    tpe: SType::SSigmaProp,
                    v: secrets.get(0).unwrap().public_image().into(),
                })));
            let output_candidates = vec![ErgoBoxCandidate::new(1u64.try_into().unwrap(), ergo_tree, 0)];
            let tx = UnsignedTransaction::new(inputs, vec![], output_candidates);

            let res = sign_transaction(prover, tx, boxes_to_spend.as_slice(), vec![].as_slice(),
                                       &ErgoStateContext::dummy());
            let signed_tx = res.unwrap();
            prop_assert!(verify_tx_proofs(&signed_tx, &boxes_to_spend).unwrap());
        }

    }
}
