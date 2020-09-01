//! Transaction signing

use super::{unsigned::UnsignedTransaction, Transaction};
use crate::{
    chain::{ergo_box::ErgoBox, ergo_state_context::ErgoStateContext, input::Input},
    eval::Env,
    sigma_protocol::prover::{Prover, ProverError},
};

/// Errors on transaction signing
pub enum TxSigningError {
    /// error on proving an input
    ProverError(ProverError, usize),
    /// failed to find an input in boxes_to_spend
    InputBoxNotFound(usize),
}

/// Signs a transaction (generating proofs for inputs)
pub fn sign_transaction(
    prover: Box<dyn Prover>,
    tx: UnsignedTransaction,
    boxes_to_spend: Vec<ErgoBox>,
    _data_boxes: Vec<ErgoBox>,
    _state_context: ErgoStateContext,
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
    #![allow(unused_imports)]
    use super::*;
    use proptest::{arbitrary::Arbitrary, collection::vec};
    use std::rc::Rc;

    // proptest! {

    // #[test]
    // fn test_tx_signing(secret in any::<DlogProverInput>()) {
    // TODO: generage a prover with multiple keys, use keys in inputs, sign the tx and verify signatures
    //     let pk = secret.public_image();
    //     let tree = ErgoTree::from(Rc::new(Expr::Const(Constant {
    //         tpe: SType::SSigmaProp,
    //         v: pk.into(),
    //     })));

    //     let prover = TestProver {
    //         secrets: vec![PrivateInput::DlogProverInput(secret)],
    //     };
    //     let res = prover.prove(&tree, &Env::empty(), message.as_slice());
    //     let proof = res.unwrap().proof;

    //     let verifier = TestVerifier;
    //     let ver_res = verifier.verify(&tree, &Env::empty(), &proof, message.as_slice());
    //     prop_assert_eq!(ver_res.unwrap().result, true);
    //
    // }

    // }
}
