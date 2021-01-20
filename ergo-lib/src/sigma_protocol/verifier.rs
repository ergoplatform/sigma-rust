//! Verifier

use std::rc::Rc;

use super::prover::ProofBytes;
use super::{
    dlog_protocol,
    fiat_shamir::{fiat_shamir_hash_fn, fiat_shamir_tree_to_bytes},
    sig_serializer::parse_sig_compute_challenges,
    unchecked_tree::{UncheckedLeaf, UncheckedSchnorr},
    SigmaBoolean, UncheckedSigmaTree, UncheckedTree,
};
use crate::ergo_tree::{ErgoTree, ErgoTreeParsingError};
use crate::eval::context::Context;
use crate::eval::env::Env;
use crate::eval::{EvalError, Evaluator};
use dlog_protocol::FirstDlogProverMessage;

/// Errors on proof verification
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VerifierError {
    /// Failed to parse ErgoTree from bytes
    ErgoTreeError(ErgoTreeParsingError),
    /// Failed to evaluate ErgoTree
    EvalError(EvalError),
}

impl From<ErgoTreeParsingError> for VerifierError {
    fn from(err: ErgoTreeParsingError) -> Self {
        VerifierError::ErgoTreeError(err)
    }
}

impl From<EvalError> for VerifierError {
    fn from(err: EvalError) -> Self {
        VerifierError::EvalError(err)
    }
}

/// Result of Box.ergoTree verification procedure (see `verify` method).
pub struct VerificationResult {
    /// result of SigmaProp condition verification via sigma protocol
    pub result: bool,
    /// estimated cost of contract execution
    pub cost: u64,
}

/// Verifier for the proofs generater by [`super::prover::Prover`]
pub trait Verifier: Evaluator {
    /// Executes the script in a given context.
    /// Step 1: Deserialize context variables
    /// Step 2: Evaluate expression and produce SigmaProp value, which is zero-knowledge statement (see also `SigmaBoolean`).
    /// Step 3: Verify that the proof is presented to satisfy SigmaProp conditions.
    fn verify(
        &self,
        tree: &ErgoTree,
        env: &Env,
        ctx: Rc<Context>,
        proof: &ProofBytes,
        message: &[u8],
    ) -> Result<VerificationResult, VerifierError> {
        let expr = tree.proposition()?;
        let cprop = self.reduce_to_crypto(expr.as_ref(), env, ctx)?.sigma_prop;
        let res: bool = match cprop {
            SigmaBoolean::TrivialProp(b) => b,
            sb => {
                // Perform Verifier Steps 1-3
                match parse_sig_compute_challenges(sb, proof) {
                    Err(_) => false,
                    Ok(UncheckedTree::UncheckedSigmaTree(sp)) => {
                        // Perform Verifier Step 4
                        let new_root = compute_commitments(sp);
                        // Verifier Steps 5-6: Convert the tree to a string `s` for input to the Fiat-Shamir hash function,
                        // using the same conversion as the prover in 7
                        // Accept the proof if the challenge at the root of the tree is equal to the Fiat-Shamir hash of `s`
                        // (and, if applicable,  the associated data). Reject otherwise.
                        let mut s = fiat_shamir_tree_to_bytes(&new_root.clone().into());
                        s.append(&mut message.to_vec());
                        let expected_challenge = fiat_shamir_hash_fn(s.as_slice());
                        new_root.challenge() == expected_challenge.into()
                    }
                    Ok(_) => todo!(),
                }
            }
        };
        Ok(VerificationResult {
            result: res,
            cost: 0,
        })
    }
}

/**
 * Verifier Step 4: For every leaf node, compute the commitment a from the challenge e and response $z$,
 * per the verifier algorithm of the leaf's Sigma-protocol.
 * If the verifier algorithm of the Sigma-protocol for any of the leaves rejects, then reject the entire proof.
 */
fn compute_commitments(sp: UncheckedSigmaTree) -> UncheckedSigmaTree {
    match sp {
        UncheckedSigmaTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(sn)) => {
            let a = dlog_protocol::interactive_prover::compute_commitment(
                &sn.proposition,
                &sn.challenge,
                &sn.second_message,
            );
            UncheckedSchnorr {
                commitment_opt: Some(FirstDlogProverMessage(a)),
                ..sn
            }
            .into()
        }
        UncheckedSigmaTree::UncheckedConjecture => todo!(),
    }
}

/// Test Verifier implementation
pub struct TestVerifier;

impl Evaluator for TestVerifier {}
impl Verifier for TestVerifier {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::constant::Constant;
    use crate::ast::expr::Expr;
    use crate::sigma_protocol::{
        private_input::{DlogProverInput, PrivateInput},
        prover::{Prover, TestProver},
    };
    use crate::types::stype::SType;
    use proptest::prelude::*;
    use std::rc::Rc;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_prover_verifier_p2pk(secret in any::<DlogProverInput>(), message in any::<Vec<u8>>()) {
            prop_assume!(!message.is_empty());
            let pk = secret.public_image();
            let tree = ErgoTree::from(Rc::new(Expr::Const(Constant {
                tpe: SType::SSigmaProp,
                v: pk.into(),
            })));

            let prover = TestProver {
                secrets: vec![PrivateInput::DlogProverInput(secret)],
            };
            let res = prover.prove(&tree, &Env::empty(), Rc::new(Context::dummy()), message.as_slice());
            let proof = res.unwrap().proof;

            let verifier = TestVerifier;
            let ver_res = verifier.verify(&tree, &Env::empty(), Rc::new(Context::dummy()),  &proof, message.as_slice());
            prop_assert_eq!(ver_res.unwrap().result, true);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn test_proof_from_mainnet() {
        use crate::chain::address::{AddressEncoder, NetworkPrefix};
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
        let message = tx.bytes_to_sign();
        let verifier = TestVerifier;
        let ver_res = verifier.verify(
            &ergo_tree,
            &Env::empty(),
            Rc::new(Context::dummy()),
            &tx.inputs.get(1).unwrap().spending_proof.proof,
            message.as_slice(),
        );
        assert_eq!(ver_res.unwrap().result, true);
    }
}
