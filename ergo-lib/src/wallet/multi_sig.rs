//! multi sig prover

use crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::transaction::unsigned::UnsignedTransaction;
use crate::chain::transaction::Transaction;
use crate::ergotree_interpreter::eval::context::Context;
use crate::ergotree_interpreter::eval::env::Env;
use crate::ergotree_interpreter::eval::reduce_to_crypto;
use crate::ergotree_interpreter::sigma_protocol::dht_protocol::interactive_prover as dht_interactive_prover;
use crate::ergotree_interpreter::sigma_protocol::dlog_protocol::interactive_prover as dlog_interactive_prover;
use crate::ergotree_interpreter::sigma_protocol::proof_tree::ProofTreeLeaf;
use crate::ergotree_interpreter::sigma_protocol::prover::hint::{
    CommitmentHint, Hint, HintsBag, OwnCommitment, RealCommitment, RealSecretProof, SecretProven,
    SimulatedCommitment, SimulatedSecretProof,
};
use crate::ergotree_interpreter::sigma_protocol::prover::ProverError::Unexpected;
use crate::ergotree_interpreter::sigma_protocol::prover::{ProofBytes, ProverError};
use crate::ergotree_interpreter::sigma_protocol::sig_serializer::SigParsingError;
use crate::ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
use crate::ergotree_interpreter::sigma_protocol::FirstProverMessage;
use crate::ergotree_ir::serialization::SigmaSerializationError;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::wallet::signing::{make_context, ErgoTransaction, TransactionContext, TxSigningError};
use ergotree_interpreter::sigma_protocol::sig_serializer::parse_sig_compute_challenges;
use ergotree_interpreter::sigma_protocol::unchecked_tree::UncheckedTree;
use ergotree_ir::chain::ergo_box::ErgoBox;
use std::collections::HashMap;
use std::rc::Rc;

/// TransactionHintsBag
pub struct TransactionHintsBag {
    secret_hints: HashMap<usize, HintsBag>,
    public_hints: HashMap<usize, HintsBag>,
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
    sigma_tree: SigmaBoolean,
    real_propositions: &[SigmaBoolean],
    simulated_propositions: &[SigmaBoolean],
    proof: &[u8],
) -> Result<HintsBag, SigParsingError> {
    let ut = parse_sig_compute_challenges(&sigma_tree, proof.to_owned())?;

    // Traversing node of sigma tree
    fn traverse_node(
        tree: UncheckedTree,
        real_propositions: &[SigmaBoolean],
        simulated_propositions: &[SigmaBoolean],
        position: NodePosition,
        bag: &mut HintsBag,
    ) {
        match tree {
            UncheckedTree::UncheckedConjecture(unchecked_conjecture) => {
                let items: SigmaConjectureItems<UncheckedTree> =
                    unchecked_conjecture.children_ust();
                items.iter().enumerate().for_each(|(i, x)| {
                    traverse_node(
                        x.clone(),
                        real_propositions,
                        simulated_propositions,
                        position.child(i),
                        bag,
                    );
                })
            }
            UncheckedTree::UncheckedLeaf(leaf) => {
                let real_found = real_propositions.contains(&leaf.proposition());
                let simulated_found = simulated_propositions.contains(&leaf.proposition());
                if real_found || simulated_found {
                    if real_found {
                        let real_commitment: Hint =
                            Hint::CommitmentHint(CommitmentHint::RealCommitment(RealCommitment {
                                image: leaf.proposition(),
                                commitment: leaf.commitment(),
                                position: position.clone(),
                            }));
                        let real_secret_proof: Hint =
                            Hint::SecretProven(SecretProven::RealSecretProof(RealSecretProof {
                                image: leaf.proposition(),
                                challenge: leaf.challenge(),
                                unchecked_tree: UncheckedTree::UncheckedLeaf(leaf),
                                position,
                            }));
                        bag.add_hint(real_commitment);
                        bag.add_hint(real_secret_proof);
                    } else {
                        let simulated_commitment: Hint = Hint::CommitmentHint(
                            CommitmentHint::SimulatedCommitment(SimulatedCommitment {
                                image: leaf.proposition(),
                                commitment: leaf.commitment(),
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
        }
    }

    let mut bag: HintsBag = HintsBag::empty();

    traverse_node(
        ut,
        real_propositions,
        simulated_propositions,
        NodePosition::crypto_tree_prefix(),
        &mut bag,
    );
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
            .get_boxes_to_spend()
            .find(|b| b.box_id() == input.box_id)
            .ok_or(TxSigningError::InputBoxNotFound(i))?;
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
        hints_bag.add_hints_for_input(i, generate_commitments_for(sigma_tree, public_keys));
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
            .get_boxes_to_spend()
            .find(|b| b.box_id() == input.box_id)
            .ok_or(TxSigningError::InputBoxNotFound(i))?;
        let height = state_context.pre_header.height;
        let self_box = tx_ctx
            .get_boxes_to_spend()
            .find(|b| b.box_id() == tx_ctx.spending_tx.inputs.as_vec()[i].box_id)
            .cloned()
            .ok_or_else(|| {
                TxSigningError::ContextError("self_index is out of bounds".to_string())
            })?;
        let outputs = tx_ctx
            .spending_tx
            .output_candidates
            .iter()
            .enumerate()
            .map(|(idx, b)| ErgoBox::from_box_candidate(b, tx_ctx.spending_tx.id(), idx as u16))
            .collect::<Result<Vec<ErgoBox>, SigmaSerializationError>>()?;
        let outputs_ir = outputs.into_iter().map(Rc::new).collect();
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
        let inputs_ir = tx_ctx
            .spending_tx
            .inputs_ids()
            .enumerated()
            .try_mapped(|(idx, u)| {
                tx_ctx
                    .boxes_to_spend
                    .iter()
                    .find(|b| u == b.box_id())
                    .map(|b| Rc::new(b.clone()))
                    .ok_or(TxSigningError::InputBoxNotFound(idx))
            })?;
        let extension = tx_ctx
            .spending_tx
            .inputs
            .get(i)
            .ok_or_else(|| {
                TxSigningError::ContextError(
                    "self_index not found in spending transaction inputs".to_string(),
                )
            })?
            .spending_proof
            .extension
            .clone();
        let ctx = Rc::new(Context {
            height,
            self_box: Rc::new(self_box),
            outputs: outputs_ir,
            data_inputs: data_inputs_ir,
            inputs: inputs_ir,
            pre_header: state_context.pre_header.clone(),
            headers: state_context.headers.clone(),
            extension,
        });
        let tree = input_box.ergo_tree.clone();
        let proof: ProofBytes = tx_ctx
            .spending_tx
            .inputs
            .get(i)
            .unwrap()
            .clone()
            .spending_proof
            .proof;
        let proof: Vec<u8> = Vec::from(proof);
        let exp = tree
            .proposition()
            .map_err(ProverError::ErgoTreeError)
            .map_err(|e| TxSigningError::ProverError(e, i))?;
        let reduction_result = reduce_to_crypto(&exp, &Env::empty(), ctx)
            .map_err(ProverError::EvalError)
            .map_err(|e| TxSigningError::ProverError(e, i))?;
        let sigma_tree = reduction_result.sigma_prop;
        let bag = bag_for_multi_sig(
            sigma_tree,
            real_secrets_to_extract.as_slice(),
            simulated_secrets_to_extract.as_slice(),
            proof.as_slice(),
        )
        .map_err(|_e| {
            TxSigningError::ProverError(Unexpected("Unexpected Sigma tree Value".to_string()), i)
        })?;
        hints_bag.add_hints_for_input(i, bag);
    }

    Ok(hints_bag)
}

/// Generating commitment for ergo-tree that is reduced to crypto
/// corresponds to an input box and with public key that prepared in input as
/// `generate_for`
pub fn generate_commitments_for(
    sigma_tree: SigmaBoolean,
    generate_for: &[SigmaBoolean],
) -> HintsBag {
    fn traverse_node(
        sb: SigmaBoolean,
        bag: &mut HintsBag,
        position: NodePosition,
        generate_for: &[SigmaBoolean],
    ) {
        let sb_clone = sb.clone();
        match sb {
            SigmaBoolean::SigmaConjecture(sc) => match sc {
                SigmaConjecture::Cand(c_and) => {
                    let items: SigmaConjectureItems<SigmaBoolean> = c_and.items;
                    items.iter().enumerate().for_each(|(i, x)| {
                        traverse_node(x.clone(), bag, position.child(i), generate_for);
                    })
                }
                SigmaConjecture::Cor(cor) => {
                    let items: SigmaConjectureItems<SigmaBoolean> = cor.items;
                    items.iter().enumerate().for_each(|(i, x)| {
                        traverse_node(x.clone(), bag, position.child(i), generate_for);
                    })
                }
                SigmaConjecture::Cthreshold(c_threshold) => {
                    let items: SigmaConjectureItems<SigmaBoolean> = c_threshold.children;
                    items.iter().enumerate().for_each(|(i, x)| {
                        traverse_node(x.clone(), bag, position.child(i), generate_for);
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
                        let (a, b) = dht_interactive_prover::first_message(&pdht);
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
    use crate::ergotree_ir::sigma_protocol::dlog_group;
    use crate::ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
    use ergo_chain_types::Base16DecodedBytes;
    use ergotree_interpreter::sigma_protocol::private_input::DhTupleProverInput;
    use ergotree_ir::mir::sigma_or::SigmaOr;
    use k256::Scalar;
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
            sigma_tree.clone(),
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
            sigma_tree,
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
                SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
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
                SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
                generate_for.as_slice(),
            )
            .hints
            .is_empty(),
            "{}",
            "{}"
        );
        assert!(
            generate_commitments_for(cand.clone(), generate_for.as_slice())
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
                SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
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
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1)),
            generate_for.as_slice(),
        );
        assert!(!bag.hints.is_empty(), "{}", "{}");
        let mut hint = bag.hints[0].clone();
        let mut a: Option<FirstProverMessage> = None;
        let mut r: Option<Scalar> = None;
        if let Hint::CommitmentHint(CommitmentHint::RealCommitment(comm)) = hint {
            assert_eq!(comm.position, NodePosition::crypto_tree_prefix());
            a = Some(comm.commitment);
        }
        hint = bag.hints[1].clone();
        if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(comm)) = hint {
            assert_eq!(comm.position, NodePosition::crypto_tree_prefix());
            r = Some(comm.secret_randomness);
        }
        let g_to_r = dlog_group::exponentiate(&dlog_group::generator(), &r.unwrap());
        assert_eq!(
            FirstProverMessage::FirstDlogProverMessage(g_to_r.into()),
            a.unwrap()
        );

        bag = generate_commitments_for(cand, generate_for.as_slice());
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
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDhTuple(pk2.clone())),
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
    fn two_party_scenario() {
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
        let hints_from_bob: HintsBag = generate_commitments_for(cand.clone(), &generate_for);
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
            bag_for_multi_sig(cand, &real_proposition, &simulated_proposition, &proof).unwrap();
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
    fn three_party_scenario() {
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
        let tree_and = ErgoTree::try_from(expr.clone()).unwrap();

        let cand = reduce_to_crypto(&expr, &Env::empty(), ctx.clone())
            .unwrap()
            .sigma_prop;
        let mut generate_for: Vec<SigmaBoolean> = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk2),
        )];
        let hints_from_bob: HintsBag = generate_commitments_for(cand.clone(), &generate_for);
        let bag2 = hints_from_bob.real_commitments();
        let bob_secret_commitment = hints_from_bob.own_commitments();
        generate_for = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDlog(pk3.clone()),
        )];
        let hints_from_carol: HintsBag = generate_commitments_for(cand.clone(), &generate_for);
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
                &tree_and,
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
            cand.clone(),
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
                &tree_and,
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
        let mut bag_b =
            bag_for_multi_sig(cand, &real_proposition, &simulated_proposition, &proof).unwrap();
        bag_b.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(
            bob_secret_commitment.first().unwrap().clone(),
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
        let proof_byte2: ProofBytes = proof2.proof;
        let verifier = TestVerifier;

        assert!(
            !verifier
                .verify(
                    &tree_and,
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
                    &tree_and,
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
                    &tree_and,
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
        let alice_hints: HintsBag = generate_commitments_for(ctree.clone(), &generate_for);
        let secret_commitment_alice = alice_hints.own_commitments();
        generate_for = vec![SigmaBoolean::ProofOfKnowledge(
            SigmaProofOfKnowledgeTree::ProveDhTuple(pk_dave.clone()),
        )];
        let dave_hints: HintsBag = generate_commitments_for(ctree.clone(), &generate_for);
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
            verifier
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
        let mut bag = bag_for_multi_sig(
            ctree,
            &real_proposition,
            &simulated_proposition,
            &proof,
        )
        .unwrap();
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
}
