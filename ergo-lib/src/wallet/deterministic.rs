use ergotree_interpreter::sigma_protocol::{
    dlog_protocol::interactive_prover::first_message_deterministic,
    private_input::PrivateInput,
    prover::{
        hint::{CommitmentHint, Hint, HintsBag, OwnCommitment, RealCommitment},
        Prover, ProverError,
    },
    unproven_tree::NodePosition,
    FirstProverMessage,
};
use ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProofOfKnowledgeTree};

pub(super) fn generate_commitments_for<P: Prover + ?Sized>(
    prover: &P,
    sigma_tree: &SigmaBoolean,
    msg: &[u8],
    aux_rand: &[u8],
) -> Result<HintsBag, ProverError> {
    let position = NodePosition::crypto_tree_prefix();
    match sigma_tree {
        SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk)) => {
            let PrivateInput::DlogProverInput(sk) = prover
                .secrets()
                .iter()
                .find(|secret| secret.public_image() == *sigma_tree)
                .ok_or(ProverError::SecretNotFound)?
                .clone()
            else {
                return Err(ProverError::SecretNotFound);
            };
            let (r, a) = first_message_deterministic(&sk, msg, aux_rand);
            let mut bag = HintsBag::empty();
            let own_commitment: Hint =
                Hint::CommitmentHint(CommitmentHint::OwnCommitment(OwnCommitment {
                    image: SigmaBoolean::ProofOfKnowledge(pk.clone().into()),
                    secret_randomness: r,
                    commitment: FirstProverMessage::FirstDlogProverMessage(a.clone()),
                    position: position.clone(),
                }));
            let real_commitment: Hint =
                Hint::CommitmentHint(CommitmentHint::RealCommitment(RealCommitment {
                    image: SigmaBoolean::ProofOfKnowledge(pk.clone().into()),
                    commitment: FirstProverMessage::FirstDlogProverMessage(a),
                    position,
                }));
            bag.add_hint(real_commitment);
            bag.add_hint(own_commitment);
            Ok(bag)
        }
        SigmaBoolean::TrivialProp(true) => Ok(HintsBag::empty()),
        SigmaBoolean::TrivialProp(false) => Err(ProverError::ReducedToFalse),
        SigmaBoolean::ProofOfKnowledge(_) | SigmaBoolean::SigmaConjecture(_) => {
            Err(ProverError::Unexpected(
                "deterministic signing requires a single ProveDlog or trivial true reduction",
            ))
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::unreachable, clippy::panic)]
mod test {
    use ergo_chain_types::EcPoint;
    use ergotree_interpreter::sigma_protocol::dlog_protocol::interactive_prover::compute_commitment;
    use ergotree_interpreter::sigma_protocol::sig_serializer::parse_sig_compute_challenges;
    use ergotree_interpreter::sigma_protocol::unchecked_tree::{UncheckedLeaf, UncheckedTree};
    use ergotree_interpreter::sigma_protocol::{private_input::DlogProverInput, wscalar::Wscalar};
    use ergotree_ir::chain::context_extension::ContextExtension;
    use ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
    use ergotree_ir::chain::ergo_box::{arbitrary::ArbBoxParameters, ErgoBox};
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    use crate::chain::ergo_box::box_builder::ErgoBoxCandidateBuilder;
    use crate::chain::transaction::unsigned::UnsignedTransaction;
    use crate::chain::transaction::{Input, Transaction, UnsignedInput};
    use crate::wallet::secret_key::SecretKey;
    use crate::wallet::signing::TransactionContext;
    use crate::wallet::Wallet;

    fn fixed_dlog(value: u8) -> SecretKey {
        let mut bytes = [0; 32];
        bytes[31] = value;
        SecretKey::dlog_from_bytes(&bytes).unwrap()
    }

    fn public_image(secret: &SecretKey) -> super::SigmaBoolean {
        super::PrivateInput::from(secret.clone()).public_image()
    }

    fn contract_context(
        sigma: super::SigmaBoolean,
    ) -> (
        TransactionContext<UnsignedTransaction>,
        crate::chain::ergo_state_context::ErgoStateContext,
    ) {
        use ergotree_ir::{
            chain::tx_id::TxId,
            mir::{constant::Constant, expr::Expr},
        };
        let expr: Expr = Constant::from(sigma).into();
        let tree = expr.try_into().unwrap();
        let candidate = ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, tree, 0)
            .build()
            .unwrap();
        let input_box = ErgoBox::from_box_candidate(&candidate, TxId::zero(), 0).unwrap();
        let tx = UnsignedTransaction::new_from_vec(
            vec![UnsignedInput::new(
                input_box.box_id(),
                ContextExtension::empty(),
            )],
            vec![],
            vec![candidate],
        )
        .unwrap();
        (
            TransactionContext::new(tx, vec![input_box], vec![]).unwrap(),
            force_any_val(),
        )
    }

    fn assert_contract_rejected(wallet: &Wallet, sigma: super::SigmaBoolean) {
        let (context, state) = contract_context(sigma);
        let reduced =
            crate::chain::transaction::reduced::reduce_tx(context.clone(), &state).unwrap();
        assert!(wallet
            .generate_deterministic_commitments(&reduced, &[])
            .is_err());
        assert!(wallet
            .sign_reduced_transaction_deterministic(reduced, &[])
            .is_err());
        assert!(wallet
            .sign_transaction_deterministic(context, &state, &[])
            .is_err());
    }

    #[test]
    fn deterministic_contract_rejects_two_key_and() {
        use ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
        let keys = vec![fixed_dlog(1), fixed_dlog(2)];
        let sigma = Cand::normalized(
            keys.iter()
                .map(public_image)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let wallet = Wallet::from_secrets(keys);
        let (context, state) = contract_context(sigma.clone());
        if let Ok(first) = wallet.sign_transaction_deterministic(context.clone(), &state, &[]) {
            let second = wallet
                .sign_transaction_deterministic(context, &state, &[])
                .unwrap();
            assert!(first.inputs.first().spending_proof.proof == second.inputs.first().spending_proof.proof,
                "unsupported two-key AND produced different proofs for identical deterministic signing inputs");
            panic!("unsupported two-key AND was signed");
        }
        assert_contract_rejected(&wallet, sigma);
    }

    #[test]
    fn deterministic_contract_rejects_dhtuple() {
        use ergotree_interpreter::sigma_protocol::private_input::DhTupleProverInput;
        use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
        let super::PrivateInput::DlogProverInput(secret) = super::PrivateInput::from(fixed_dlog(3))
        else {
            unreachable!()
        };
        let generator = ergo_chain_types::ec_point::generator();
        let point = *secret.public_image().h;
        let key = SecretKey::DhtSecretKey(DhTupleProverInput {
            w: secret.w,
            common_input: ProveDhTuple::new(generator, generator, point, point),
        });
        let sigma = public_image(&key);
        assert_contract_rejected(&Wallet::from_secrets(vec![key]), sigma);
    }

    #[test]
    fn deterministic_contract_rejects_missing_key() {
        assert_contract_rejected(&Wallet::from_secrets(vec![]), public_image(&fixed_dlog(1)));
    }

    #[test]
    fn deterministic_contract_rejects_false() {
        assert_contract_rejected(&Wallet::from_secrets(vec![]), false.into());
    }

    #[test]
    fn deterministic_contract_single_dlog_is_repeatable() {
        let key = fixed_dlog(1);
        let sigma = public_image(&key);
        let (context, state) = contract_context(sigma.clone());
        let message = context.spending_tx.bytes_to_sign().unwrap();
        let wallet = Wallet::from_secrets(vec![key]);
        let reduced =
            crate::chain::transaction::reduced::reduce_tx(context.clone(), &state).unwrap();
        let first = wallet
            .sign_transaction_deterministic(context.clone(), &state, &[])
            .unwrap();
        let second = wallet
            .sign_transaction_deterministic(context, &state, &[])
            .unwrap();
        let from_reduced = wallet
            .sign_reduced_transaction_deterministic(reduced, &[])
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(first, from_reduced);
        assert!(!first
            .inputs
            .first()
            .spending_proof
            .proof
            .clone()
            .to_bytes()
            .is_empty());
        assert!(
            ergotree_interpreter::sigma_protocol::verifier::verify_signature(
                sigma,
                &message,
                &first.inputs.first().spending_proof.proof.clone().to_bytes(),
            )
            .unwrap()
        );
    }

    #[test]
    fn deterministic_contract_true_needs_no_witness() {
        let (context, state) = contract_context(true.into());
        let wallet = Wallet::from_secrets(vec![]);
        let reduced =
            crate::chain::transaction::reduced::reduce_tx(context.clone(), &state).unwrap();
        let first = wallet
            .sign_transaction_deterministic(context, &state, &[])
            .unwrap();
        let from_reduced = wallet
            .sign_reduced_transaction_deterministic(reduced, &[])
            .unwrap();
        assert_eq!(first, from_reduced);
        assert!(first
            .inputs
            .first()
            .spending_proof
            .proof
            .clone()
            .to_bytes()
            .is_empty());
    }

    fn gen_boxes() -> impl Strategy<Value = (SecretKey, Vec<ErgoBox>)> {
        any::<Wscalar>()
            .prop_map(|s| SecretKey::DlogSecretKey(DlogProverInput::new(s)))
            .prop_flat_map(|sk: SecretKey| {
                (
                    Just(sk.clone()),
                    vec(
                        any_with::<ErgoBox>(ArbBoxParameters {
                            ergo_tree: Just(sk.get_address_from_public_image().script().unwrap())
                                .boxed(),
                            registers: Just(NonMandatoryRegisters::empty()).boxed(),
                            tokens: Just(None).boxed(),
                            ..Default::default()
                        }),
                        1..10,
                    ),
                )
            })
    }

    fn parse_sig(sk: &SecretKey, input: &Input) -> (EcPoint, Vec<u8>) {
        let ergotree_ir::chain::address::Address::P2Pk(pk) = sk.get_address_from_public_image()
        else {
            unreachable!()
        };
        let UncheckedTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(schnorr)) =
            parse_sig_compute_challenges(
                &SigmaProofOfKnowledgeTree::from(pk.clone()).into(),
                input.spending_proof.proof.clone().to_bytes(),
            )
            .unwrap()
        else {
            unreachable!();
        };
        let commitment = compute_commitment(&pk, &schnorr.challenge, &schnorr.second_message);
        (commitment, schnorr.challenge.into())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]
        // Produce signatures for different messages and test for nonce re-use
        #[test]
        fn test_sign_deterministic((sk, boxes) in gen_boxes()) {
            let wallet = Wallet::from_secrets(vec![sk.clone()]);
            let output = ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, sk.get_address_from_public_image().script().unwrap(), 0).build().unwrap();
            let inputs: Vec<_> = boxes.iter().map(|b| UnsignedInput::new(b.box_id(), ContextExtension::empty())).collect();
            let txes: Vec<Transaction> = (1..10).map(|i| {
                let mut output = output.clone();
                output.value = output.value.checked_mul_u32(i).unwrap();
                let tx = UnsignedTransaction::new_from_vec(inputs.clone(), vec![], vec![output]).unwrap();
                wallet.sign_transaction_deterministic(TransactionContext::new(tx, boxes.clone(), vec![]).unwrap(), &force_any_val(), &[]).unwrap()
            }).collect();
            let signatures: Vec<_> = txes.iter().flat_map(|tx| tx.inputs.iter()).map(|input| parse_sig(&sk, input)).collect();

            for (i, (r, c)) in signatures.iter().enumerate() {
                if let Some((_, _)) = signatures.iter().enumerate().find(|(j, (r1, c1))| i != *j && r1 == r && c != c1) {
                    panic!();
                }
            }

        }
    }
}
