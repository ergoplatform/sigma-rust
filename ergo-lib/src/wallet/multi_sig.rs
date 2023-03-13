//! multi sig prove crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::transaction::unsigned::UnsignedTransaction;
use crate::chain::transaction::Transaction;
use crate::ergotree_interpreter::eval::env::Env;
use crate::ergotree_interpreter::eval::reduce_to_crypto;
use crate::ergotree_interpreter::sigma_protocol::dht_protocol::interactive_prover as dht_interactive_prover;
use crate::ergotree_interpreter::sigma_protocol::dlog_protocol::interactive_prover as dlog_interactive_prover;
use crate::ergotree_interpreter::sigma_protocol::proof_tree::ProofTreeLeaf;
use crate::ergotree_interpreter::sigma_protocol::prover::hint::{
    CommitmentHint, Hint, HintsBag, OwnCommitment, RealCommitment, RealSecretProof, SecretProven,
    SimulatedCommitment, SimulatedSecretProof,
};
use crate::ergotree_interpreter::sigma_protocol::prover::ProverError;
use crate::ergotree_interpreter::sigma_protocol::sig_serializer::SigParsingError;
use crate::ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
use crate::ergotree_interpreter::sigma_protocol::FirstProverMessage;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::wallet::signing::{make_context, TransactionContext, TxSigningError};
use ergotree_interpreter::sigma_protocol::sig_serializer::parse_sig_compute_challenges;
use ergotree_interpreter::sigma_protocol::unchecked_tree::UncheckedTree;
use ergotree_interpreter::sigma_protocol::verifier::compute_commitments;
use std::collections::HashMap;
use std::rc::Rc;

use super::tx_context::TransactionContextError;

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
            strategy = "proptest::collection::hash_map(proptest::prelude::any::<usize>(), proptest::prelude::any::<HintsBag>(), 0..5)"
        )
    )]
    pub(crate) secret_hints: HashMap<usize, HintsBag>,
    #[cfg_attr(
        feature = "arbitrary",
        proptest(
            strategy = "proptest::collection::hash_map(proptest::prelude::any::<usize>(), proptest::prelude::any::<HintsBag>(), 0..5)"
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

    /// Replacing Hints for an input index
    pub fn replace_hints_for_input(&mut self, index: usize, hints_bag: HintsBag) {
        let public: Vec<Hint> = hints_bag
            .hints
            .clone()
            .into_iter()
            .filter(|hint| matches!(hint, Hint::CommitmentHint(_)))
            .collect();
        let secret: Vec<Hint> = hints_bag
            .hints
            .into_iter()
            .filter(|hint| matches!(hint, Hint::SecretProven(_)))
            .collect();

        self.secret_hints.insert(index, HintsBag { hints: secret });
        self.public_hints.insert(index, HintsBag { hints: public });
    }

    /// Adding hints for a input index
    pub fn add_hints_for_input(&mut self, index: usize, hints_bag: HintsBag) {
        let mut public: Vec<Hint> = hints_bag
            .hints
            .clone()
            .into_iter()
            .filter(|hint| matches!(hint, Hint::CommitmentHint(_)))
            .collect();
        let mut secret: Vec<Hint> = hints_bag
            .hints
            .into_iter()
            .filter(|hint| matches!(hint, Hint::SecretProven(_)))
            .collect();
        let secret_bag = HintsBag::empty();
        let public_bag = HintsBag::empty();
        let old_secret: &Vec<Hint> = &self.secret_hints.get(&index).unwrap_or(&secret_bag).hints;
        for hint in old_secret {
            secret.push(hint.clone());
        }

        let old_public: &Vec<Hint> = &self.public_hints.get(&index).unwrap_or(&public_bag).hints;
        for hint in old_public {
            public.push(hint.clone());
        }
        self.secret_hints.insert(index, HintsBag { hints: secret });
        self.public_hints.insert(index, HintsBag { hints: public });
    }

    /// Outputting HintsBag corresponding for an index
    pub fn all_hints_for_input(&self, index: usize) -> HintsBag {
        let mut hints: Vec<Hint> = Vec::new();
        let secret_bag = HintsBag::empty();
        let public_bag = HintsBag::empty();
        let secrets: &Vec<Hint> = &self.secret_hints.get(&index).unwrap_or(&secret_bag).hints;
        for hint in secrets {
            hints.push(hint.clone());
        }
        let public: &Vec<Hint> = &self.public_hints.get(&index).unwrap_or(&public_bag).hints;
        for hint in public {
            hints.push(hint.clone());
        }
        let hints_bag: HintsBag = HintsBag { hints };
        hints_bag
    }
}

/// A method which is extracting partial proofs of secret knowledge for particular secrets with their
/// respective public images given. Useful for distributed signature applications.
/// See DistributedSigSpecification for examples of usage.
pub fn bag_for_multi_sig(
    sigma_tree: &SigmaBoolean,
    real_propositions: &[SigmaBoolean],
    simulated_propositions: &[SigmaBoolean],
    proof: &[u8],
) -> Result<HintsBag, SigParsingError> {
    if let SigmaBoolean::TrivialProp(_) = sigma_tree {
        return Ok(HintsBag::empty());
    }
    let ut = compute_commitments(parse_sig_compute_challenges(sigma_tree, proof.to_owned())?);
    // Traversing node of sigma tree
    fn traverse_node(
        tree: UncheckedTree,
        real_propositions: &[SigmaBoolean],
        simulated_propositions: &[SigmaBoolean],
        position: NodePosition,
        bag: &mut HintsBag,
    ) -> Result<(), SigParsingError> {
        match tree {
            UncheckedTree::UncheckedConjecture(unchecked_conjecture) => {
                let items: SigmaConjectureItems<UncheckedTree> =
                    unchecked_conjecture.children_ust();
                items
                    .iter()
                    .enumerate()
                    .try_for_each(|(i, x)| -> Result<(), SigParsingError> {
                        traverse_node(
                            x.clone(),
                            real_propositions,
                            simulated_propositions,
                            position.child(i),
                            bag,
                        )?;
                        Ok(())
                    })?;
            }
            UncheckedTree::UncheckedLeaf(leaf) => {
                match leaf.commitment_opt() {
                    Some(commitment) => {
                        let real_found = real_propositions.contains(&leaf.proposition());
                        let simulated_found = simulated_propositions.contains(&leaf.proposition());
                        if real_found || simulated_found {
                            if real_found {
                                let real_commitment: Hint = Hint::CommitmentHint(
                                    CommitmentHint::RealCommitment(RealCommitment {
                                        image: leaf.proposition(),
                                        commitment,
                                        position: position.clone(),
                                    }),
                                );
                                let real_secret_proof: Hint = Hint::SecretProven(
                                    SecretProven::RealSecretProof(RealSecretProof {
                                        image: leaf.proposition(),
                                        challenge: leaf.challenge(),
                                        unchecked_tree: UncheckedTree::UncheckedLeaf(leaf),
                                        position,
                                    }),
                                );
                                bag.add_hint(real_commitment);
                                bag.add_hint(real_secret_proof);
                            } else {
                                let simulated_commitment: Hint = Hint::CommitmentHint(
                                    CommitmentHint::SimulatedCommitment(SimulatedCommitment {
                                        image: leaf.proposition(),
                                        commitment,
                                        position: position.clone(),
                                    }),
                                );
                                let simulated_secret_proof: Hint = Hint::SecretProven(
                                    SecretProven::SimulatedSecretProof(SimulatedSecretProof {
                                        image: leaf.proposition(),
                                        challenge: leaf.challenge(),
                                        unchecked_tree: UncheckedTree::UncheckedLeaf(leaf),
                                        position,
                                    }),
                                );
                                bag.add_hint(simulated_commitment);
                                bag.add_hint(simulated_secret_proof);
                            }
                        }
                    }
                    None => {
                        return Err(SigParsingError::EmptyCommitment(leaf.proposition()));
                    }
                };
            }
        }
        Ok(())
    }

    let mut bag: HintsBag = HintsBag::empty();

    traverse_node(
        ut,
        real_propositions,
        simulated_propositions,
        NodePosition::crypto_tree_prefix(),
        &mut bag,
    )?;
    Ok(bag)
}

/// A method which is generating commitments to randomness. A commitment is about a first step
/// of a zero-knowledge proof-of-knowledge knowledge protocol.
/// the commitments are generated for every input box of transaction
/// and return as `TransactionHintsBag`.
/// generated commitments corresponds to `public_keys` that prepared in function input
pub fn generate_commitments(
    tx_context: TransactionContext<UnsignedTransaction>,
    state_context: &ErgoStateContext,
    public_keys: &[SigmaBoolean],
) -> Result<TransactionHintsBag, TxSigningError> {
    let tx = tx_context.spending_tx.clone();
    let mut hints_bag = TransactionHintsBag::empty();
    for (i, input) in tx.inputs.iter().enumerate() {
        let input_box = tx_context
            .get_input_box(&input.box_id)
            .ok_or(TransactionContextError::InputBoxNotFound(i))?;
        let ctx = Rc::new(make_context(state_context, &tx_context, i)?);
        let tree = input_box.ergo_tree.clone();
        let exp = tree
            .proposition()
            .map_err(ProverError::ErgoTreeError)
            .map_err(|e| TxSigningError::ProverError(e, i))?;
        let reduction_result = reduce_to_crypto(&exp, &Env::empty(), ctx)
            .map_err(ProverError::EvalError)
            .map_err(|e| TxSigningError::ProverError(e, i))?;

        let sigma_tree = reduction_result.sigma_prop;
        hints_bag.add_hints_for_input(i, generate_commitments_for(&sigma_tree, public_keys));
    }
    Ok(hints_bag)
}

/// Extracting hints from a transaction and outputs it's corresponding TransactionHintsBag
pub fn extract_hints(
    tx_ctx: &TransactionContext<Transaction>,
    state_context: &ErgoStateContext,
    real_secrets_to_extract: Vec<SigmaBoolean>,
    simulated_secrets_to_extract: Vec<SigmaBoolean>,
) -> Result<TransactionHintsBag, TxSigningError> {
    let mut hints_bag = TransactionHintsBag::empty();
    for (i, input) in tx_ctx.spending_tx.inputs.iter().enumerate() {
        let input_box = tx_ctx
            .get_input_box(&input.box_id)
            .ok_or(TransactionContextError::InputBoxNotFound(i))?;
        let ctx = Rc::new(make_context(state_context, tx_ctx, i)?);
        let tree = input_box.ergo_tree.clone();
        let exp = tree
            .proposition()
            .map_err(ProverError::ErgoTreeError)
            .map_err(|e| TxSigningError::ProverError(e, i))?;
        let reduction_result = reduce_to_crypto(&exp, &Env::empty(), ctx)
            .map_err(ProverError::EvalError)
            .map_err(|e| TxSigningError::ProverError(e, i))?;
        let sigma_tree = reduction_result.sigma_prop;
        let bag = bag_for_multi_sig(
            &sigma_tree,
            real_secrets_to_extract.as_slice(),
            simulated_secrets_to_extract.as_slice(),
            input.spending_proof.proof.as_ref(),
        )?;
        hints_bag.add_hints_for_input(i, bag);
    }

    Ok(hints_bag)
}

/// Generating commitment for ergo-tree that is reduced to crypto
/// corresponds to an input box and with public key that prepared in input as
/// `generate_for`
pub fn generate_commitments_for(
    sigma_tree: &SigmaBoolean,
    generate_for: &[SigmaBoolean],
) -> HintsBag {
    fn traverse_node(
        sb: &SigmaBoolean,
        bag: &mut HintsBag,
        position: NodePosition,
        generate_for: &[SigmaBoolean],
    ) {
        let sb_clone = sb.clone();
        match sb {
            SigmaBoolean::SigmaConjecture(sc) => match sc {
                SigmaConjecture::Cand(c_and) => {
                    c_and.items.iter().enumerate().for_each(|(i, x)| {
                        traverse_node(x, bag, position.child(i), generate_for);
                    })
                }
                SigmaConjecture::Cor(cor) => cor.items.iter().enumerate().for_each(|(i, x)| {
                    traverse_node(x, bag, position.child(i), generate_for);
                }),
                SigmaConjecture::Cthreshold(c_threshold) => {
                    c_threshold.children.iter().enumerate().for_each(|(i, x)| {
                        traverse_node(x, bag, position.child(i), generate_for);
                    })
                }
            },
            SigmaBoolean::ProofOfKnowledge(kt) => {
                if generate_for.contains(&sb_clone) {
                    let kt_clone = kt.clone();
                    if let SigmaProofOfKnowledgeTree::ProveDlog(_pdl) = kt {
                        let (r, a) = dlog_interactive_prover::first_message();
                        let own_commitment: Hint =
                            Hint::CommitmentHint(CommitmentHint::OwnCommitment(OwnCommitment {
                                image: SigmaBoolean::ProofOfKnowledge(kt_clone.clone()),
                                secret_randomness: r,
                                commitment: FirstProverMessage::FirstDlogProverMessage(a.clone()),
                                position: position.clone(),
                            }));
                        let real_commitment: Hint =
                            Hint::CommitmentHint(CommitmentHint::RealCommitment(RealCommitment {
                                image: SigmaBoolean::ProofOfKnowledge(kt_clone),
                                commitment: FirstProverMessage::FirstDlogProverMessage(a),
                                position,
                            }));
                        bag.add_hint(real_commitment);
                        bag.add_hint(own_commitment);
                    } else if let SigmaProofOfKnowledgeTree::ProveDhTuple(pdht) = kt {
                        let (a, b) = dht_interactive_prover::first_message(pdht);
                        let own_commitment: Hint =
                            Hint::CommitmentHint(CommitmentHint::OwnCommitment(OwnCommitment {
                                image: SigmaBoolean::ProofOfKnowledge(kt_clone.clone()),
                                secret_randomness: a,
                                commitment: FirstProverMessage::FirstDhtProverMessage(b.clone()),
                                position: position.clone(),
                            }));
                        let real_commitment: Hint =
                            Hint::CommitmentHint(CommitmentHint::RealCommitment(RealCommitment {
                                image: SigmaBoolean::ProofOfKnowledge(kt_clone),
                                commitment: FirstProverMessage::FirstDhtProverMessage(b),
                                position,
                            }));
                        bag.add_hint(real_commitment);
                        bag.add_hint(own_commitment);
                    }
                }
            }
            _ => (),
        }
    }
    let mut bag = HintsBag::empty();
    traverse_node(
        sigma_tree,
        &mut bag,
        NodePosition::crypto_tree_prefix(),
        generate_for,
    );
    bag
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::transaction::Transaction;
    use crate::ergotree_interpreter::eval::context::Context;
    use crate::ergotree_interpreter::eval::env::Env;
    use crate::ergotree_interpreter::eval::reduce_to_crypto;
    use crate::ergotree_interpreter::sigma_protocol::private_input::{
        DlogProverInput, PrivateInput,
    };
    use crate::ergotree_interpreter::sigma_protocol::prover::{ProofBytes, Prover, TestProver};
    use crate::ergotree_interpreter::sigma_protocol::verifier::{TestVerifier, Verifier};
    use crate::ergotree_ir::chain::address::AddressEncoder;
    use crate::ergotree_ir::chain::address::{Address, NetworkPrefix};
    use crate::ergotree_ir::ergo_tree::ErgoTree;
    use crate::ergotree_ir::mir::expr::Expr;
    use crate::ergotree_ir::mir::sigma_and::SigmaAnd;
    use crate::ergotree_ir::serialization::SigmaSerializable;
    use crate::ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
    use ergo_chain_types::Base16DecodedBytes;
    use ergotree_interpreter::sigma_protocol::private_input::DhTupleProverInput;
    use ergotree_interpreter::sigma_protocol::wscalar::Wscalar;
    use ergotree_ir::mir::atleast::Atleast;
    use ergotree_ir::mir::constant::{Constant, Literal};
    use ergotree_ir::mir::sigma_or::SigmaOr;
    use ergotree_ir::mir::value::CollKind;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;
    use std::convert::{TryFrom, TryInto};
    use std::rc::Rc;

    #[test]
    fn extract_hint() {
        let signed_tx = r#"{
          "id": "6e32d1710816be34fd9710148b73f017bf8e71115cd2d3cf5758f80c2e3010ca",
          "inputs": [
            {
              "boxId": "4c1155fca9bf7785f82eb43f74ef6a24164bac18f6cc35137e0ebf5a08abb8f7",
              "spendingProof": {
                "proofBytes": "77d85667cbb360c7ddad4d94c5e50930f3e72f6bdbd576bcce0098ab6547221d6943a49d4f6daaf06b37efc29884337701c16f4a0b9797db3061332b09849274beaa0609f146a468937338792bfe422425dd604b399df221",
                "extension": {}
              }
            }
          ],
          "dataInputs": [],
          "outputs": [
            {
              "boxId": "ad3e07a89bd0ec1161c1da54316ceb8efc6734ed08d3f005ade1184bf26d088d",
              "value": 1000000,
              "ergoTree": "0008cd039c8404d33f85dd4012e4f3d0719951eeea0015b13b940d67d4990e13de28b154",
              "assets": [],
              "additionalRegisters": {},
              "creationHeight": 0,
              "transactionId": "6e32d1710816be34fd9710148b73f017bf8e71115cd2d3cf5758f80c2e3010ca",
              "index": 0
            },
            {
              "boxId": "bcf15dbfd2b7d5e4688cb28d1393356bd1c96d6ef94c2942a2958545a51d2501",
              "value": 2300000,
              "ergoTree": "0008cd039c8404d33f85dd4012e4f3d0719951eeea0015b13b940d67d4990e13de28b154",
              "assets": [],
              "additionalRegisters": {},
              "creationHeight": 0,
              "transactionId": "6e32d1710816be34fd9710148b73f017bf8e71115cd2d3cf5758f80c2e3010ca",
              "index": 1
            },
            {
              "boxId": "6e473151a782e68cff7fd4f0127eeb43cf71e7a6d7fddf1b25ad4814fb451292",
              "value": 1100000,
              "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
              "assets": [],
              "additionalRegisters": {},
              "creationHeight": 0,
              "transactionId": "6e32d1710816be34fd9710148b73f017bf8e71115cd2d3cf5758f80c2e3010ca",
              "index": 2
            }
          ]
        }"#;
        let bytes_m = Base16DecodedBytes::try_from("100208cd03c847c306a2f9a8087b4ae63261cc5acea9034000ba8d033b0fb033247e8aade908cd02f4b05f44eb9703db7fcf9c94b89566787a7188c7e48964821d485d9ef2f9e4c4ea0273007301").unwrap();
        let tree_m: ErgoTree = ErgoTree::sigma_parse_bytes(&bytes_m.0).unwrap();

        let contx = Rc::new(force_any_val::<Context>());
        let exp = tree_m.proposition().unwrap();
        let reduction_result = reduce_to_crypto(&exp, &Env::empty(), contx).unwrap();
        let sigma_tree = reduction_result.sigma_prop;
        let stx: Transaction = serde_json::from_str(signed_tx).unwrap();
        let test: ProofBytes = stx.inputs.first().clone().spending_proof.proof;
        let proof: Vec<u8> = Vec::from(test);
        let mut real_proposition: Vec<SigmaBoolean> = Vec::new();
        let mut simulated_proposition: Vec<SigmaBoolean> = Vec::new();
        let mut bag: HintsBag = bag_for_multi_sig(
            &sigma_tree,
            real_proposition.as_slice(),
            simulated_proposition.as_slice(),
            proof.as_slice(),
        )
        .unwrap();
        assert!(bag.hints.is_empty(), "{}", "{}");

        let address_encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let first_address: Address = address_encoder
            .parse_address_from_str("9hz1anoLGZGf88hjjrQ3y3oSpSE2Kkk15CcfFqCNYXDPWKV6F4i")
            .unwrap();
        let second_address = address_encoder
            .parse_address_from_str("9gNpmNsivNnAKb7EtGVLGF24wRUJWnEqycCnWCeYxwSFZxkKyTZ")
            .unwrap();

        if let Address::P2Pk(pk) = first_address {
            {
                real_proposition.push(SigmaBoolean::ProofOfKnowledge(
                    SigmaProofOfKnowledgeTree::ProveDlog(pk),
                ));
            }
        }
        if let Address::P2Pk(pk) = second_address {
            {
                simulated_proposition.push(SigmaBoolean::ProofOfKnowledge(
                    SigmaProofOfKnowledgeTree::ProveDlog(pk),
                ));
            }
        }
        bag = bag_for_multi_sig(
            &sigma_tree,
            real_proposition.as_slice(),
            simulated_proposition.as_slice(),
            proof.as_slice(),
        )
        .unwrap();
        assert!(!bag.hints.is_empty(), "{}", "{}");
    }

    #[test]
    fn generating_commitment_two_signer() {
        let secret1 = DlogProverInput::random();
        let secret2 = DhTupleProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let mut generate_for: Vec<SigmaBoolean> = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDhTuple(pk2.clone()),
        )];

        assert_eq!(
            generate_commitments_for(
                &SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
                generate_for.as_slice(),
            )
            .hints
            .len(),
            0
        );
        generate_for.clear();
        let cand = Cand::normalized(
            vec![
                pk1.clone().into(),
                SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDhTuple(
                    pk2.clone(),
                )),
            ]
            .try_into()
            .unwrap(),
        );
        generate_for.push(cand.clone());
        assert!(
            generate_commitments_for(
                &SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
                generate_for.as_slice(),
            )
            .hints
            .is_empty(),
            "{}",
            "{}"
        );
        assert!(
            generate_commitments_for(&cand, generate_for.as_slice())
                .hints
                .is_empty(),
            "{}",
            "{}"
        );
        generate_for.clear();
        generate_for.push(SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone()),
        ));

        assert!(
            !generate_commitments_for(
                &SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
                generate_for.as_slice(),
            )
            .hints
            .is_empty(),
            "{}",
            "{}"
        );
        generate_for.clear();
        generate_for.push(SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone()),
        ));
        let mut bag = generate_commitments_for(
            &SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1)),
            generate_for.as_slice(),
        );
        assert!(!bag.hints.is_empty(), "{}", "{}");
        let mut hint = bag.hints[0].clone();
        let mut a: Option<FirstProverMessage> = None;
        let mut r: Option<Wscalar> = None;
        if let Hint::CommitmentHint(CommitmentHint::RealCommitment(comm)) = hint {
            assert_eq!(comm.position, NodePosition::crypto_tree_prefix());
            a = Some(comm.commitment);
        }
        hint = bag.hints[1].clone();
        if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(comm)) = hint {
            assert_eq!(comm.position, NodePosition::crypto_tree_prefix());
            r = Some(comm.secret_randomness);
        }
        use ergo_chain_types::ec_point::{exponentiate, generator};
        let g_to_r = exponentiate(&generator(), r.unwrap().as_scalar_ref());
        assert_eq!(
            FirstProverMessage::FirstDlogProverMessage(g_to_r.into()),
            a.unwrap()
        );

        bag = generate_commitments_for(&cand, generate_for.as_slice());
        assert_eq!(bag.hints.len(), 2);
        hint = bag.hints[0].clone();
        if let Hint::CommitmentHint(CommitmentHint::RealCommitment(comm)) = hint {
            assert_eq!(
                comm.position,
                NodePosition {
                    positions: vec![0, 0]
                }
            );
        }
        hint = bag.hints[1].clone();
        if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(comm)) = hint {
            assert_eq!(
                comm.position,
                NodePosition {
                    positions: vec![0, 0]
                }
            );
        }
        generate_for.clear();
        generate_for.push(SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDhTuple(pk2.clone()),
        ));
        bag = generate_commitments_for(
            &SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDhTuple(pk2.clone())),
            generate_for.as_slice(),
        );
        assert_ne!(bag.hints.len(), 0);
        hint = bag.hints[0].clone();
        if let Hint::CommitmentHint(CommitmentHint::RealCommitment(comm)) = hint {
            assert_eq!(comm.position, NodePosition::crypto_tree_prefix(),);
        }
        hint = bag.hints[1].clone();
        if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(comm)) = hint {
            assert_eq!(comm.position, NodePosition::crypto_tree_prefix(),);
        }
        hint = bag.hints[0].clone();
        if let Hint::CommitmentHint(CommitmentHint::RealCommitment(real_commitment)) = hint {
            if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(own_commitment)) =
                bag.hints[1].clone()
            {
                assert_eq!(real_commitment.commitment, own_commitment.commitment,);
            }
        }
    }

    #[test]
    fn multi_sig_2() {
        let ctx = Rc::new(force_any_val::<Context>());

        let secret1 = DlogProverInput::random();
        let secret2 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let prover1 = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret1)],
        };
        let prover2 = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret2)],
        };
        let expr: Expr = SigmaAnd::new(vec![
            Expr::Const(pk1.clone().into()),
            Expr::Const(pk2.clone().into()),
        ])
        .unwrap()
        .into();
        let tree_and = ErgoTree::try_from(expr.clone()).unwrap();

        let cand = reduce_to_crypto(&expr, &Env::empty(), ctx.clone())
            .unwrap()
            .sigma_prop;
        let generate_for: Vec<SigmaBoolean> = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk2),
        )];
        let hints_from_bob: HintsBag = generate_commitments_for(&cand, &generate_for);
        let bag1 = hints_from_bob.real_commitments();
        let own = hints_from_bob.own_commitments();
        let message = vec![0u8; 100];
        let mut bag_a = HintsBag { hints: vec![] };
        bag_a.add_hint(Hint::CommitmentHint(CommitmentHint::RealCommitment(
            bag1.first().unwrap().clone(),
        )));

        let proof1 = prover1
            .prove(
                &tree_and,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_a,
            )
            .unwrap();
        let proof: Vec<u8> = Vec::from(proof1.proof);
        let real_proposition: Vec<SigmaBoolean> = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk1),
        )];
        let simulated_proposition: Vec<SigmaBoolean> = Vec::new();
        let mut bag_b =
            bag_for_multi_sig(&cand, &real_proposition, &simulated_proposition, &proof).unwrap();
        bag_b.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(
            own.first().unwrap().clone(),
        )));
        let proof2 = prover2
            .prove(
                &tree_and,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_b,
            )
            .unwrap();
        let proof_byte: ProofBytes = proof2.proof;
        let verifier = TestVerifier;

        assert!(
            verifier
                .verify(
                    &tree_and,
                    &Env::empty(),
                    ctx,
                    proof_byte,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "{}",
            "{}"
        );
    }

    #[test]
    fn multi_sig_and_3() {
        let ctx = Rc::new(force_any_val::<Context>());

        let secret1 = DlogProverInput::random();
        let secret2 = DlogProverInput::random();
        let secret3 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let pk3 = secret3.public_image();
        let prover1 = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret1)],
        };
        let prover2 = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret2)],
        };
        let prover3 = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret3)],
        };

        let expr: Expr = SigmaAnd::new(vec![
            Expr::Const(pk1.clone().into()),
            Expr::Const(pk2.clone().into()),
            Expr::Const(pk3.clone().into()),
        ])
        .unwrap()
        .into();

        let tree_expr = ErgoTree::try_from(expr.clone()).unwrap();

        let expr_reduced = reduce_to_crypto(&expr, &Env::empty(), ctx.clone())
            .unwrap()
            .sigma_prop;
        let mut generate_for: Vec<SigmaBoolean> = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk2),
        )];
        let hints_from_bob: HintsBag = generate_commitments_for(&expr_reduced, &generate_for);
        let bag2 = hints_from_bob.real_commitments();
        let bob_secret_commitment = hints_from_bob.own_commitments();
        generate_for = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk3.clone()),
        )];
        let hints_from_carol: HintsBag = generate_commitments_for(&expr_reduced, &generate_for);
        let bag3 = hints_from_carol.real_commitments();
        let carol_secret_commitment = hints_from_carol.own_commitments();
        let message = vec![0u8; 100];
        let mut bag_a = HintsBag { hints: vec![] };
        bag_a.add_hint(Hint::CommitmentHint(CommitmentHint::RealCommitment(
            bag2.first().unwrap().clone(),
        )));
        bag_a.add_hint(Hint::CommitmentHint(CommitmentHint::RealCommitment(
            bag3.first().unwrap().clone(),
        )));
        let proof1 = prover1
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_a,
            )
            .unwrap();
        let mut proof: Vec<u8> = Vec::from(proof1.proof.clone());
        let proof_byte1: ProofBytes = proof1.proof;
        let mut real_proposition: Vec<SigmaBoolean> = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone()),
        )];
        let simulated_proposition: Vec<SigmaBoolean> = Vec::new();
        let mut bag_c = bag_for_multi_sig(
            &expr_reduced,
            &real_proposition,
            &simulated_proposition,
            &proof,
        )
        .unwrap();
        bag_c.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(
            carol_secret_commitment.first().unwrap().clone(),
        )));
        bag_c.add_hint(Hint::CommitmentHint(CommitmentHint::RealCommitment(
            bag2.first().unwrap().clone(),
        )));
        let proof3 = prover3
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_c,
            )
            .unwrap();
        proof = Vec::from(proof3.proof.clone());
        let proof_byte3: ProofBytes = proof3.proof;
        real_proposition = vec![
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1)),
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk3)),
        ];
        let mut bag_b = bag_for_multi_sig(
            &expr_reduced,
            &real_proposition,
            &simulated_proposition,
            &proof,
        )
        .unwrap();
        bag_b.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(
            bob_secret_commitment.first().unwrap().clone(),
        )));
        let proof2 = prover2
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_b,
            )
            .unwrap();
        let proof_byte2: ProofBytes = proof2.proof;
        let verifier = TestVerifier;

        assert!(
            !verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    proof_byte1,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "{}",
            "{}"
        );

        assert!(
            !verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    proof_byte3,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "{}",
            "{}"
        );

        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx,
                    proof_byte2,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "{}",
            "{}"
        );
    }

    #[test]
    fn multi_dlog_dht() {
        let ctx = Rc::new(force_any_val::<Context>());

        let secret_alice = DlogProverInput::random();
        let secret_bob = DlogProverInput::random();
        let secret_carol = DhTupleProverInput::random();
        let secret_dave = DhTupleProverInput::random();

        let pk_alice = secret_alice.public_image();
        let pk_bob = secret_bob.public_image();
        let pk_carol = secret_carol.public_image();
        let pk_dave = secret_dave.public_image();
        let prover_a = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret_alice)],
        };
        let _prover_b = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret_bob)],
        };
        let _prover_c = TestProver {
            secrets: vec![PrivateInput::DhTupleProverInput(secret_carol.clone())],
        };
        let _prover_d = TestProver {
            secrets: vec![PrivateInput::DhTupleProverInput(secret_dave.clone())],
        };
        let first_expr: Expr = SigmaOr::new(vec![
            Expr::Const(pk_alice.clone().into()),
            Expr::Const(pk_bob.clone().into()),
        ])
        .unwrap()
        .into();
        let second_expr: Expr = SigmaOr::new(vec![
            Expr::Const(pk_carol.clone().into()),
            Expr::Const(pk_dave.clone().into()),
        ])
        .unwrap()
        .into();
        let exp: Expr = SigmaAnd::new(vec![first_expr, second_expr]).unwrap().into();
        let tree = ErgoTree::try_from(exp.clone()).unwrap();
        let ctree = reduce_to_crypto(&exp, &Env::empty(), ctx.clone())
            .unwrap()
            .sigma_prop;
        let mut generate_for: Vec<SigmaBoolean> = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk_alice.clone()),
        )];
        let alice_hints: HintsBag = generate_commitments_for(&ctree, &generate_for);
        let secret_commitment_alice = alice_hints.own_commitments();
        generate_for = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDhTuple(pk_dave.clone()),
        )];
        let dave_hints: HintsBag = generate_commitments_for(&ctree, &generate_for);
        let dave_known = dave_hints.real_commitments();
        let _dave_secret_commitment = dave_hints.own_commitments();
        let message = vec![0u8; 100];
        let mut bag_a = HintsBag::empty();
        bag_a.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(
            secret_commitment_alice.first().unwrap().clone(),
        )));
        bag_a.add_hint(Hint::CommitmentHint(CommitmentHint::RealCommitment(
            dave_known.first().unwrap().clone(),
        )));
        let proof_a = prover_a
            .prove(
                &tree,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_a,
            )
            .unwrap();
        let proof: Vec<u8> = Vec::from(proof_a.proof.clone());
        let proof_byte_a: ProofBytes = proof_a.proof;
        let verifier = TestVerifier;

        assert!(
            !verifier
                .verify(
                    &tree,
                    &Env::empty(),
                    ctx.clone(),
                    proof_byte_a,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "{}",
            "{}"
        );
        let real_proposition: Vec<SigmaBoolean> = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk_alice),
        )];
        let simulated_proposition: Vec<SigmaBoolean> = vec![
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk_bob.clone())),
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDhTuple(
                pk_carol.clone(),
            )),
        ];
        println!(
            "{:?}",
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk_bob))
        );
        let mut bag =
            bag_for_multi_sig(&ctree, &real_proposition, &simulated_proposition, &proof).unwrap();
        bag.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(
            _dave_secret_commitment.first().unwrap().clone(),
        )));

        let proof_d = _prover_d
            .prove(&tree, &Env::empty(), ctx.clone(), message.as_slice(), &bag)
            .unwrap();
        let proof_byte_d: ProofBytes = proof_d.proof;

        assert!(
            verifier
                .verify(&tree, &Env::empty(), ctx, proof_byte_d, message.as_slice(),)
                .unwrap()
                .result,
            "{}",
            "{}"
        );
    }

    #[test]
    fn multi_sig_atleast_2_out_of_3() {
        // from https://github.com/ScorexFoundation/sigmastate-interpreter/blob/78dd1e715038c2f95c518fb56977c6591b76e20c/sc/src/test/scala/sigmastate/utxo/DistributedSigSpecification.scala#L124
        let ctx = Rc::new(force_any_val::<Context>());

        let alice_secret = DlogProverInput::random();
        let bob_secret = DlogProverInput::random();
        let carol_secret = DlogProverInput::random();
        let alice_pk = alice_secret.public_image();
        let bob_pk = bob_secret.public_image();
        let carol_pk = carol_secret.public_image();
        let alice_prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(alice_secret)],
        };
        let bob_prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(bob_secret)],
        };
        let _carol_prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(carol_secret)],
        };

        let bound = Expr::Const(2i32.into());
        let inputs = Literal::Coll(
            CollKind::from_vec(
                SType::SSigmaProp,
                vec![
                    SigmaProp::from(alice_pk.clone()).into(),
                    SigmaProp::from(bob_pk.clone()).into(),
                    SigmaProp::from(carol_pk.clone()).into(),
                ],
            )
            .unwrap(),
        );
        let input = Constant {
            tpe: SType::SColl(SType::SSigmaProp.into()),
            v: inputs,
        }
        .into();
        let expr: Expr = Atleast::new(bound, input).unwrap().into();

        let tree_expr = ErgoTree::try_from(expr.clone()).unwrap();

        let expr_reduced = reduce_to_crypto(&expr, &Env::empty(), ctx.clone())
            .unwrap()
            .sigma_prop;
        let message = vec![0u8; 100];

        let hints_from_bob: HintsBag = generate_commitments_for(&expr_reduced, &[bob_pk.into()]);

        let bob_real_commitment = hints_from_bob.real_commitments();

        let mut bag_a = HintsBag { hints: vec![] };
        bag_a.add_hint(Hint::CommitmentHint(CommitmentHint::RealCommitment(
            bob_real_commitment.first().unwrap().clone(),
        )));

        let proof_alice = alice_prover
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_a,
            )
            .unwrap();

        let mut bag_b = bag_for_multi_sig(
            &expr_reduced,
            &[alice_pk.into()],
            &[carol_pk.into()],
            proof_alice.proof.as_ref(),
        )
        .unwrap();
        bag_b.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(
            hints_from_bob.own_commitments().first().unwrap().clone(),
        )));

        let proof_bob = bob_prover
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_b,
            )
            .unwrap();

        let verifier = TestVerifier;

        assert!(
            !verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    proof_alice.proof,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "Proof generated by Alice without getting Bob's part is not correct"
        );

        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx,
                    proof_bob.proof,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "Compound proof from Bob is correct"
        );
    }

    #[test]
    fn multi_sig_atleast_3_out_of_4() {
        // from https://github.com/ScorexFoundation/sigmastate-interpreter/blob/78dd1e715038c2f95c518fb56977c6591b76e20c/sc/src/test/scala/sigmastate/utxo/DistributedSigSpecification.scala#L160-L205

        let ctx = Rc::new(force_any_val::<Context>());

        let alice_secret = DlogProverInput::random();
        let bob_secret = DlogProverInput::random();
        let carol_secret = DlogProverInput::random();
        let dave_secret = DlogProverInput::random();
        let alice_pk = alice_secret.public_image();
        let bob_pk = bob_secret.public_image();
        let carol_pk = carol_secret.public_image();
        let dave_pk = dave_secret.public_image();
        let alice_prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(alice_secret)],
        };
        let bob_prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(bob_secret)],
        };
        let _carol_prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(carol_secret)],
        };

        let bound = Expr::Const(3i32.into());
        let inputs = Literal::Coll(
            CollKind::from_vec(
                SType::SSigmaProp,
                vec![
                    SigmaProp::from(alice_pk.clone()).into(),
                    SigmaProp::from(bob_pk.clone()).into(),
                    SigmaProp::from(carol_pk.clone()).into(),
                    SigmaProp::from(dave_pk.clone()).into(),
                ],
            )
            .unwrap(),
        );
        let input = Constant {
            tpe: SType::SColl(SType::SSigmaProp.into()),
            v: inputs,
        }
        .into();
        let expr: Expr = Atleast::new(bound, input).unwrap().into();

        let tree_expr = ErgoTree::try_from(expr.clone()).unwrap();

        let expr_reduced = reduce_to_crypto(&expr, &Env::empty(), ctx.clone())
            .unwrap()
            .sigma_prop;
        let message = vec![0u8; 100];

        let bob_hints: HintsBag = generate_commitments_for(&expr_reduced, &[bob_pk.into()]);
        let dl_b_known = bob_hints.real_commitments().first().unwrap().clone();

        let carol_hints: HintsBag =
            generate_commitments_for(&expr_reduced, &[carol_pk.clone().into()]);
        let dl_c_known = carol_hints.real_commitments().first().unwrap().clone();

        let bag_a = HintsBag {
            hints: vec![dl_b_known.clone().into(), dl_c_known.into()],
        };

        let proof_alice = alice_prover
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_a,
            )
            .unwrap();

        let mut bag_c = bag_for_multi_sig(
            &expr_reduced,
            &[alice_pk.clone().into()],
            &[dave_pk.clone().into()],
            proof_alice.proof.as_ref(),
        )
        .unwrap();
        bag_c.hints.push(dl_b_known.into());
        bag_c.hints.push(
            carol_hints
                .own_commitments()
                .first()
                .unwrap()
                .clone()
                .into(),
        );

        let proof_carol = _carol_prover
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_c,
            )
            .unwrap();

        let bag_b_1 = bag_for_multi_sig(
            &expr_reduced,
            &[alice_pk.into()],
            &[dave_pk.into()],
            proof_alice.proof.as_ref(),
        )
        .unwrap();

        let bag_b_2 = bag_for_multi_sig(
            &expr_reduced,
            &[carol_pk.into()],
            &[],
            proof_carol.proof.as_ref(),
        )
        .unwrap();

        let mut bag_b = HintsBag::from_bags(vec![bag_b_1, bag_b_2]);

        bag_b.add_hint(bob_hints.own_commitments().first().unwrap().clone().into());

        let proof_bob = bob_prover
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                message.as_slice(),
                &bag_b,
            )
            .unwrap();

        let verifier = TestVerifier;

        assert!(
            !verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    proof_alice.proof,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "Proof generated by Alice without getting Bob's part is not correct"
        );

        assert!(
            !verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    proof_carol.proof,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "Proof generated by Carol without getting Bob's part is not correct"
        );

        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx,
                    proof_bob.proof,
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "Compound proof from Bob is correct"
        );
    }

    #[test]
    fn multi_sig_atleast_7_out_of_10_i692() {
        // based on
        // https://github.com/ScorexFoundation/sigmastate-interpreter/blob/78dd1e715038c2f95c518fb56977c6591b76e20c/sc/src/test/scala/sigmastate/utxo/DistributedSigSpecification.scala#L299-L389
        let ctx = Rc::new(force_any_val::<Context>());

        let sk1 = DlogProverInput::random();
        let pk1 = sk1.public_image();
        let sk2 = DlogProverInput::random();
        let pk2 = sk2.public_image();
        let sk3 = DlogProverInput::random();
        let pk3 = sk3.public_image();
        let sk4 = DlogProverInput::random();
        let pk4 = sk4.public_image();
        let sk5 = DlogProverInput::random();
        let pk5 = sk5.public_image();
        let sk6 = DlogProverInput::random();
        let pk6 = sk6.public_image();
        let sk7 = DlogProverInput::random();
        let pk7 = sk7.public_image();
        let sk8 = DlogProverInput::random();
        let pk8 = sk8.public_image();
        let sk9 = DlogProverInput::random();
        let pk9 = sk9.public_image();
        let sk10 = DlogProverInput::random();
        let pk10 = sk10.public_image();

        let prover1 = TestProver {
            secrets: vec![sk1.into()],
        };
        let prover2 = TestProver {
            secrets: vec![sk2.into()],
        };
        let prover3 = TestProver {
            secrets: vec![sk3.into()],
        };
        let prover4 = TestProver {
            secrets: vec![sk4.into()],
        };
        let prover5 = TestProver {
            secrets: vec![sk5.into()],
        };
        let prover6 = TestProver {
            secrets: vec![sk6.into()],
        };
        let prover7 = TestProver {
            secrets: vec![sk7.into()],
        };
        // let prover8 = TestProver {
        //     secrets: vec![sk8.into()],
        // };
        // let prover9 = TestProver {
        //     secrets: vec![sk9.into()],
        // };
        // let prover10 = TestProver {
        //     secrets: vec![sk10.into()],
        // };

        let bound = Expr::Const(7i32.into());
        let input = Constant {
            tpe: SType::SColl(SType::SSigmaProp.into()),
            v: Literal::Coll(
                CollKind::from_vec(
                    SType::SSigmaProp,
                    vec![
                        SigmaProp::from(pk1.clone()).into(),
                        SigmaProp::from(pk2.clone()).into(),
                        SigmaProp::from(pk3.clone()).into(),
                        SigmaProp::from(pk4.clone()).into(),
                        SigmaProp::from(pk5.clone()).into(),
                        SigmaProp::from(pk6.clone()).into(),
                        SigmaProp::from(pk7.clone()).into(),
                        SigmaProp::from(pk8.clone()).into(),
                        SigmaProp::from(pk9.clone()).into(),
                        SigmaProp::from(pk10.clone()).into(),
                    ],
                )
                .unwrap(),
            ),
        }
        .into();
        let expr: Expr = Atleast::new(bound, input).unwrap().into();
        let tree_expr = ErgoTree::try_from(expr.clone()).unwrap();
        let expr_reduced = reduce_to_crypto(&expr, &Env::empty(), ctx.clone())
            .unwrap()
            .sigma_prop;
        let message = vec![0u8; 100];

        // only actors 1, 2, 3, 4, 5, 6, 7 are signing, others are simulated (see bag_one below)

        let hints_1 = generate_commitments_for(&expr_reduced, &[pk1.clone().into()]);
        let dl_1_known = hints_1.real_commitments().first().unwrap().clone();
        let secret_cmt_1 = hints_1.own_commitments().first().unwrap().clone();

        let hints_2 = generate_commitments_for(&expr_reduced, &[pk2.clone().into()]);
        let dl_2_known = hints_2.real_commitments().first().unwrap().clone();
        let secret_cmt_2 = hints_2.own_commitments().first().unwrap().clone();

        let hints_3 = generate_commitments_for(&expr_reduced, &[pk3.clone().into()]);
        let dl_3_known = hints_3.real_commitments().first().unwrap().clone();
        let secret_cmt_3 = hints_3.own_commitments().first().unwrap().clone();

        let hints_4 = generate_commitments_for(&expr_reduced, &[pk4.clone().into()]);
        let dl_4_known = hints_4.real_commitments().first().unwrap().clone();
        let secret_cmt_4 = hints_4.own_commitments().first().unwrap().clone();

        let hints_5 = generate_commitments_for(&expr_reduced, &[pk5.clone().into()]);
        let dl_5_known = hints_5.real_commitments().first().unwrap().clone();
        let secret_cmt_5 = hints_5.own_commitments().first().unwrap().clone();

        let hints_6 = generate_commitments_for(&expr_reduced, &[pk6.clone().into()]);
        let dl_6_known = hints_6.real_commitments().first().unwrap().clone();
        let secret_cmt_6 = hints_6.own_commitments().first().unwrap().clone();

        let hints_7 = generate_commitments_for(&expr_reduced, &[pk7.clone().into()]);
        let secret_cmt_7 = hints_7.own_commitments().first().unwrap().clone();

        let bag_7 = HintsBag {
            hints: vec![
                dl_1_known.clone().into(),
                dl_2_known.clone().into(),
                dl_3_known.clone().into(),
                dl_4_known.clone().into(),
                dl_5_known.clone().into(),
                dl_6_known.clone().into(),
                secret_cmt_7.clone().into(),
            ],
        };

        let proof_7 = prover7
            .prove(&tree_expr, &Env::empty(), ctx.clone(), &message, &bag_7)
            .unwrap();

        let verifier = TestVerifier;

        assert!(
            !verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    proof_7.proof.clone(),
                    message.as_slice(),
                )
                .unwrap()
                .result,
            "Proof generated by Prover7 only is not correct"
        );

        //hints after the first real proof done.
        let bag_one = bag_for_multi_sig(
            &expr_reduced,
            &[pk7.into()],
            &[pk8.into(), pk9.into(), pk10.into()],
            proof_7.proof.as_ref(),
        )
        .unwrap();

        //now real proofs can be done in any order
        let mut bag_2 = bag_one.clone();
        bag_2.add_hint(secret_cmt_2.clone().into());
        bag_2.add_hint(dl_1_known.clone().into());
        bag_2.add_hint(dl_3_known.clone().into());
        bag_2.add_hint(dl_4_known.clone().into());
        bag_2.add_hint(dl_5_known.clone().into());
        bag_2.add_hint(dl_6_known.clone().into());
        let proof_2 = prover2
            .prove(&tree_expr, &Env::empty(), ctx.clone(), &message, &bag_2)
            .unwrap();
        let partial_proof_2 =
            bag_for_multi_sig(&expr_reduced, &[pk2.into()], &[], proof_2.proof.as_ref())
                .unwrap()
                .real_proofs()
                .first()
                .unwrap()
                .clone();

        let mut bag_1 = bag_one.clone();
        bag_1.add_hint(secret_cmt_1.clone().into());
        bag_1.add_hint(dl_2_known.clone().into());
        bag_1.add_hint(dl_3_known.clone().into());
        bag_1.add_hint(dl_4_known.clone().into());
        bag_1.add_hint(dl_5_known.clone().into());
        bag_1.add_hint(dl_6_known.clone().into());

        let proof_1 = prover1
            .prove(&tree_expr, &Env::empty(), ctx.clone(), &message, &bag_1)
            .unwrap();
        let partial_proof_1 =
            bag_for_multi_sig(&expr_reduced, &[pk1.into()], &[], proof_1.proof.as_ref())
                .unwrap()
                .real_proofs()
                .first()
                .unwrap()
                .clone();

        let mut bag_3 = bag_one.clone();
        bag_3.add_hint(secret_cmt_3.clone().into());
        bag_3.add_hint(dl_1_known.clone().into());
        bag_3.add_hint(dl_2_known.clone().into());
        bag_3.add_hint(dl_4_known.clone().into());
        bag_3.add_hint(dl_5_known.clone().into());
        bag_3.add_hint(dl_6_known.clone().into());
        let proof_3 = prover3
            .prove(&tree_expr, &Env::empty(), ctx.clone(), &message, &bag_3)
            .unwrap();
        let partial_proof_3 =
            bag_for_multi_sig(&expr_reduced, &[pk3.into()], &[], proof_3.proof.as_ref())
                .unwrap()
                .real_proofs()
                .first()
                .unwrap()
                .clone();

        let mut bag_4 = bag_one.clone();
        bag_4.add_hint(secret_cmt_4.clone().into());
        bag_4.add_hint(dl_1_known.clone().into());
        bag_4.add_hint(dl_2_known.clone().into());
        bag_4.add_hint(dl_3_known.clone().into());
        bag_4.add_hint(dl_5_known.clone().into());
        bag_4.add_hint(dl_6_known.clone().into());
        let proof_4 = prover4
            .prove(&tree_expr, &Env::empty(), ctx.clone(), &message, &bag_4)
            .unwrap();
        let partial_proof_4 =
            bag_for_multi_sig(&expr_reduced, &[pk4.into()], &[], proof_4.proof.as_ref())
                .unwrap()
                .real_proofs()
                .first()
                .unwrap()
                .clone();

        let mut bag_5 = bag_one.clone();
        bag_5.add_hint(secret_cmt_5.clone().into());
        bag_5.add_hint(dl_1_known.clone().into());
        bag_5.add_hint(dl_2_known.clone().into());
        bag_5.add_hint(dl_3_known.clone().into());
        bag_5.add_hint(dl_4_known.clone().into());
        bag_5.add_hint(dl_6_known.clone().into());
        let proof_5 = prover5
            .prove(&tree_expr, &Env::empty(), ctx.clone(), &message, &bag_5)
            .unwrap();
        let partial_proof_5 =
            bag_for_multi_sig(&expr_reduced, &[pk5.into()], &[], proof_5.proof.as_ref())
                .unwrap()
                .real_proofs()
                .first()
                .unwrap()
                .clone();

        let mut bag_6 = bag_one.clone();
        bag_6.add_hint(secret_cmt_6.clone().into());
        bag_6.add_hint(dl_1_known.clone().into());
        bag_6.add_hint(dl_2_known.clone().into());
        bag_6.add_hint(dl_3_known.clone().into());
        bag_6.add_hint(dl_4_known.clone().into());
        bag_6.add_hint(dl_5_known.clone().into());
        let proof_6 = prover6
            .prove(&tree_expr, &Env::empty(), ctx.clone(), &message, &bag_6)
            .unwrap();
        let partial_proof_6 =
            bag_for_multi_sig(&expr_reduced, &[pk6.into()], &[], proof_6.proof.as_ref())
                .unwrap()
                .real_proofs()
                .first()
                .unwrap()
                .clone();

        let mut bag = bag_one;
        bag.add_hint(partial_proof_1.into());
        bag.add_hint(partial_proof_2.into());
        bag.add_hint(partial_proof_3.into());
        bag.add_hint(partial_proof_4.into());
        bag.add_hint(partial_proof_5.into());
        bag.add_hint(partial_proof_6.into());
        bag.add_hint(dl_1_known.into());
        bag.add_hint(dl_2_known.into());
        bag.add_hint(dl_3_known.into());
        bag.add_hint(dl_4_known.into());
        bag.add_hint(dl_5_known.into());
        bag.add_hint(dl_6_known.into());

        let mut valid_bag_1 = bag.clone();
        valid_bag_1.add_hint(secret_cmt_1.into());
        let valid_proof_1 = prover1
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                &message,
                &valid_bag_1,
            )
            .unwrap();

        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    valid_proof_1.proof.clone(),
                    message.as_slice(),
                )
                .unwrap()
                .result,
        );

        let mut valid_bag_2 = bag.clone();
        valid_bag_2.add_hint(secret_cmt_2.into());
        let valid_proof_2 = prover2
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                &message,
                &valid_bag_2,
            )
            .unwrap();
        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    valid_proof_2.proof.clone(),
                    message.as_slice(),
                )
                .unwrap()
                .result,
        );

        let mut valid_bag_3 = bag.clone();
        valid_bag_3.add_hint(secret_cmt_3.into());
        let valid_proof_3 = prover3
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                &message,
                &valid_bag_3,
            )
            .unwrap();
        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    valid_proof_3.proof.clone(),
                    message.as_slice()
                )
                .unwrap()
                .result
        );

        let mut valid_bag_4 = bag.clone();
        valid_bag_4.add_hint(secret_cmt_4.into());
        let valid_proof_4 = prover4
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                &message,
                &valid_bag_4,
            )
            .unwrap();
        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    valid_proof_4.proof.clone(),
                    message.as_slice()
                )
                .unwrap()
                .result
        );

        let mut valid_bag_5 = bag.clone();
        valid_bag_5.add_hint(secret_cmt_5.into());
        let valid_proof_5 = prover5
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                &message,
                &valid_bag_5,
            )
            .unwrap();
        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    valid_proof_5.proof.clone(),
                    message.as_slice()
                )
                .unwrap()
                .result
        );

        let mut valid_bag_6 = bag.clone();
        valid_bag_6.add_hint(secret_cmt_6.into());
        let valid_proof_6 = prover6
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                &message,
                &valid_bag_6,
            )
            .unwrap();
        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx.clone(),
                    valid_proof_6.proof.clone(),
                    message.as_slice()
                )
                .unwrap()
                .result
        );

        let mut valid_bag_7 = bag.clone();
        valid_bag_7.add_hint(secret_cmt_7.into());
        let valid_proof_7 = prover7
            .prove(
                &tree_expr,
                &Env::empty(),
                ctx.clone(),
                &message,
                &valid_bag_7,
            )
            .unwrap();
        assert!(
            verifier
                .verify(
                    &tree_expr,
                    &Env::empty(),
                    ctx,
                    valid_proof_7.proof.clone(),
                    message.as_slice()
                )
                .unwrap()
                .result
        );

        assert_eq!(valid_proof_1.proof, valid_proof_2.proof);
        assert_eq!(valid_proof_2.proof, valid_proof_3.proof);
        assert_eq!(valid_proof_3.proof, valid_proof_4.proof);
        assert_eq!(valid_proof_4.proof, valid_proof_5.proof);
        assert_eq!(valid_proof_5.proof, valid_proof_6.proof);
        assert_eq!(valid_proof_6.proof, valid_proof_7.proof);
    }
}
