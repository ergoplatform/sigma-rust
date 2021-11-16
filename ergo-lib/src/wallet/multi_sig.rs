//! multi sig prover

use crate::chain::transaction::reduced::{reduce_tx, ReducedTransaction, ReducedInput};
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::{SigmaProofOfKnowledgeTree, ProveDlog};
use crate::ergotree_interpreter::sigma_protocol::dlog_protocol::{FirstDlogProverMessage, interactive_prover};
use crate::ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
use crate::ergotree_interpreter::sigma_protocol::prover::hint::{OwnCommitment, RealCommitment, HintsBag, Hint, CommitmentHint, SecretProven, RealSecretProof, SimulatedCommitment, SimulatedSecretProof};
use crate::ergotree_interpreter::sigma_protocol::FirstProverMessage;
use crate::ergotree_ir::chain::address::{AddressEncoder, Address};
use crate::ergotree_ir::chain::address::NetworkPrefix;
use std::collections::HashMap;
use std::rc::Rc;
use crate::chain::transaction::{Transaction, TxIoVec};
use crate::wallet::signing::{make_context, TransactionContext};
use crate::chain::ergo_state_context::ErgoStateContext;
use ergotree_interpreter::sigma_protocol::unchecked_tree::{UncheckedConjecture, UncheckedLeaf, UncheckedTree};
use ergotree_interpreter::sigma_protocol::dlog_protocol::{interactive_prover::compute_commitment};
use crate::ergotree_interpreter::sigma_protocol::proof_tree::ProofTreeLeaf;
use ergotree_interpreter::sigma_protocol::sig_serializer::parse_sig_compute_challenges;
use crate::ergotree_interpreter::eval::env::Env;
use crate::ergotree_interpreter::eval::reduce_to_crypto;
use crate::ergotree_interpreter::sigma_protocol::prover::ProofBytes;

pub struct TransactionHintsBag {
    secret_hints: HashMap<usize, HintsBag>,
    public_hints: HashMap<usize, HintsBag>,
}

impl TransactionHintsBag {

    pub fn empty() -> Self {
        TransactionHintsBag { secret_hints: HashMap::new(), public_hints: HashMap::new() }
    }

    pub fn replace_hints_for_input(&mut self, index: usize, hints_bag: HintsBag) {
        let public: Vec<Hint> = hints_bag.hints.clone().into_iter().filter(|hint| {
            if let Hint::CommitmentHint(_) = hint {
                true
            } else {
                false
            }
        }).collect();
        let secret: Vec<Hint> = hints_bag.hints.clone().into_iter().filter(|hint| {
            if let Hint::SecretProven(_) = hint {
                true
            } else {
                false
            }
        }).collect();

        self.secret_hints.insert(index, HintsBag { hints: secret });
        self.public_hints.insert(index, HintsBag { hints: public });
    }
    pub fn add_hints_for_input(&mut self, index: usize, hints_bag: HintsBag) {
        let mut public: Vec<Hint> = hints_bag.hints.clone().into_iter().filter(|hint| {
            if let Hint::CommitmentHint(_) = hint {
                true
            } else {
                false
            }
            // if let Hint::CommitmentHint(CommitmentHint::RealCommitment(comm)) = hint {
            //     true
            // }
            // else if let Hint::CommitmentHint(CommitmentHint::SimulatedCommitment(comm)) = hint{
            //     true
            // } else {
            //     false
            // }
        }).collect();
        let mut secret: Vec<Hint> = hints_bag.hints.clone().into_iter().filter(|hint| {
            if let Hint::SecretProven(_) = hint {
                true
            } else {
                false
            }
            // if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(comm)) = hint {
            //     true
            // } else {
            //     false
            // }
        }).collect();
        let mut secret_bag = HintsBag::empty();
        let mut public_bag = HintsBag::empty();
        let mut old_secret: &Vec<Hint> = &self.secret_hints.get(&index).unwrap_or(&secret_bag).hints;
        for hint in old_secret {
            secret.push(hint.clone());
        }


        let mut old_public: &Vec<Hint> = &self.public_hints.get(&index).unwrap_or(&public_bag).hints;
        for hint in old_public {
            public.push(hint.clone());
        }
        self.secret_hints.insert(index, HintsBag { hints: secret });
        self.public_hints.insert(index, HintsBag { hints: public });

    }
    pub fn all_hints_for_input(&self, index: usize) -> HintsBag {
        let mut hints: Vec<Hint> = Vec::new();
        let mut secret_bag = HintsBag::empty();
        let mut public_bag = HintsBag::empty();
        let mut secrets: &Vec<Hint> = &self.secret_hints.get(&index).unwrap_or(&secret_bag).hints;
        for hint in secrets {
            hints.push(hint.clone());
        }
        let mut public: &Vec<Hint> = &self.public_hints.get(&index).unwrap_or(&public_bag).hints;
        for hint in public {
            hints.push(hint.clone());
        }
        let hints_bag: HintsBag = HintsBag { hints };
        return hints_bag;
    }
}

pub fn compute_commitments(leaf:UncheckedLeaf)->Option<FirstDlogProverMessage>{
    let mut ret:Option<FirstDlogProverMessage>=None;

    match leaf{
        UncheckedLeaf::UncheckedSchnorr(pdlog) => {
            let challenge=pdlog.challenge;
            let proposition=pdlog.proposition;
            let second_message=pdlog.second_message;
            let comm=compute_commitment(&proposition,&challenge,&second_message);
            ret=Some(FirstDlogProverMessage::from(comm));
        }
        UncheckedLeaf::UncheckedDhTuple(_) => {}
    }
    return ret;
}

pub fn bag_for_multi_sig(
    sigma_tree: SigmaBoolean,
    real_propositions: &Vec<SigmaBoolean>,
    simulated_propositions: &Vec<SigmaBoolean>,
    proof:&Vec<u8>,
) ->HintsBag{
    let ut:UncheckedTree=parse_sig_compute_challenges(&sigma_tree,proof.clone()).unwrap();
    let mut bag:HintsBag = HintsBag::empty();
    traverse_node(ut, real_propositions, simulated_propositions, NodePosition::crypto_tree_prefix().clone(), &mut bag);
    return bag;

}

pub fn extract_hints(
    signed_tx:Transaction,
    tx_context: TransactionContext,
    state_context: &ErgoStateContext,
    real_secrets_to_extract: Vec<SigmaBoolean>,
    simulated_secrets_to_extract: Vec<SigmaBoolean>)
->TransactionHintsBag{
    let mut hints_bag: TransactionHintsBag = TransactionHintsBag { secret_hints: HashMap::new(), public_hints: HashMap::new() };

    tx_context
        .get_boxes_to_spend().enumerate().for_each(|(i, input)|{
        let ctx = Rc::new(make_context(state_context, &tx_context, i).unwrap());
        let tree=input.ergo_tree.clone();
        let test:ProofBytes=signed_tx.inputs.get(i).unwrap().clone().spending_proof.proof;
        let proof:Vec<u8>=Vec::from(test);
        let exp=tree.proposition().unwrap();
        let reduction_result=reduce_to_crypto(&exp,&Env::empty(),ctx.clone()).unwrap();
        let sigma_tree=reduction_result.sigma_prop;
        hints_bag.add_hints_for_input(
            i,
            bag_for_multi_sig(
                sigma_tree.clone(),
                &real_secrets_to_extract,
                &simulated_secrets_to_extract,
                &proof
            )
        );

    });
    return hints_bag;

}

pub fn traverse_node(
    tree:UncheckedTree,
    real_propositions: &Vec<SigmaBoolean>,
    simulated_propositions:&Vec<SigmaBoolean>,
    position:NodePosition,
    bag: &mut HintsBag
){
    println!("in traverse");
    match tree{
        UncheckedTree::UncheckedConjecture(unchecked_conjecture) => {
            let items: SigmaConjectureItems<UncheckedTree> = unchecked_conjecture.children_ust();
            items.iter().enumerate().for_each(|(i, x)| {
                traverse_node(
                    x.clone(),
                    real_propositions,
                    simulated_propositions,
                    position.child(i),
                    bag
                );
            })
        }
        UncheckedTree::UncheckedLeaf(leaf) => {
            let real_found= real_propositions.contains(&leaf.proposition());
            let simulated_found= simulated_propositions.contains(&leaf.proposition());
            // println!("{}{}",real_found,simulated_found);
            if real_found || simulated_found {
                let a=compute_commitments(leaf.clone()).unwrap();
                if real_found{
                    let real_commitment: Hint = Hint::CommitmentHint(
                        CommitmentHint::RealCommitment(
                            RealCommitment {
                                image: leaf.proposition().clone(),
                                commitment: FirstProverMessage::FirstDlogProverMessage(
                                    a.clone()
                                ),
                                position: position.clone(),
                            }
                        ));
                    let real_secret_proof:Hint=Hint::SecretProven(
                        SecretProven::RealSecretProof(
                            RealSecretProof{
                                image:leaf.proposition().clone(),
                                challenge:leaf.challenge().clone(),
                                unchecked_tree:UncheckedTree::UncheckedLeaf(leaf.clone()),
                                position:position.clone(),
                            }
                        )
                    );
                    bag.add_hint(real_commitment);
                    bag.add_hint(real_secret_proof);
                } else {
                    let simulated_commitment: Hint = Hint::CommitmentHint(
                        CommitmentHint::SimulatedCommitment(
                            SimulatedCommitment {
                                image: leaf.proposition().clone(),
                                commitment: FirstProverMessage::FirstDlogProverMessage(
                                    a.clone()
                                ),
                                position: position.clone(),
                            }
                        ));
                    let simulated_secret_proof:Hint=Hint::SecretProven(
                        SecretProven::SimulatedSecretProof(
                            SimulatedSecretProof{
                                image:leaf.proposition().clone(),
                                challenge:leaf.challenge().clone(),
                                unchecked_tree:UncheckedTree::UncheckedLeaf(leaf.clone()),
                                position:position.clone()
                            }
                        )
                    );
                    bag.add_hint(simulated_commitment);
                    bag.add_hint(simulated_secret_proof);
                }

            }


        }
    }
}

pub fn generate_commitments_for(
    sigmatree: SigmaBoolean,
    generate_for: &Vec<SigmaBoolean>,
) -> HintsBag {
    fn traverse_node(
        sb: SigmaBoolean,
        bag: &mut HintsBag,
        position: NodePosition,
        generate_for: &Vec<SigmaBoolean>,
    ) {
        let sb_clone = sb.clone();
        match sb {
            SigmaBoolean::SigmaConjecture(sc) => {
                match sc {
                    SigmaConjecture::Cand(cand) => {
                        let items: SigmaConjectureItems<SigmaBoolean> = cand.items;
                        items.iter().enumerate().for_each(|(i, x)| {
                            traverse_node(
                                x.clone(),
                                bag,
                                position.child(i),
                                generate_for,
                            );
                        })
                    }
                    SigmaConjecture::Cor(cor) => {
                        let items: SigmaConjectureItems<SigmaBoolean> = cor.items;
                        items.iter().enumerate().for_each(|(i, x)| {
                            traverse_node(
                                x.clone(),
                                bag,
                                position.child(i),
                                generate_for,
                            );
                        })
                    }
                    SigmaConjecture::Cthreshold(cthresh) => {
                        let items: SigmaConjectureItems<SigmaBoolean> = cthresh.items;
                        items.iter().enumerate().for_each(|(i, x)| {
                            traverse_node(
                                x.clone(),
                                bag,
                                position.child(i),
                                generate_for,
                            );
                        })
                    }
                }
            }
            SigmaBoolean::ProofOfKnowledge(kt) => {
                if generate_for.contains(&sb_clone) {
                    let kt_clone = kt.clone();
                    match kt {
                        SigmaProofOfKnowledgeTree::ProveDlog(_pdl) => {
                            let (r, a) = interactive_prover::first_message();
                            let owncmt: Hint = Hint::CommitmentHint(
                                CommitmentHint::OwnCommitment(
                                    OwnCommitment {
                                        image: SigmaBoolean::ProofOfKnowledge(kt_clone.clone()),
                                        secret_randomness: r,
                                        commitment: FirstProverMessage::FirstDlogProverMessage(
                                            a.clone()
                                        ),
                                        position: position.clone(),
                                    }
                                ));
                            let realcmt: Hint = Hint::CommitmentHint(
                                CommitmentHint::RealCommitment(
                                    RealCommitment {
                                        image: SigmaBoolean::ProofOfKnowledge(kt_clone.clone()),
                                        commitment: FirstProverMessage::FirstDlogProverMessage(
                                            a.clone()
                                        ),
                                        position: position.clone(),
                                    }
                                ));
                            // let mut test=HintsBag::empty();
                            bag.add_hint(realcmt);
                            bag.add_hint(owncmt);
                        }
                        /// prove dhtuple should be implemented
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
    let mut bag = HintsBag::empty();
    traverse_node(
        sigmatree,
        &mut bag,
        NodePosition::crypto_tree_prefix().clone(),
        generate_for,
    );
    return bag;
}

#[cfg(test)]
mod tests{
    use std::rc::Rc;
    use super::*;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;
    use crate::chain::transaction::{Transaction, TxId};
    use crate::ergotree_interpreter::eval::context::Context;
    use crate::ergotree_interpreter::eval::env::Env;
    use crate::ergotree_interpreter::eval::{reduce_to_crypto, ReductionResult};
    use crate::ergotree_interpreter::sigma_protocol::prover::{ProofBytes, Prover, TestProver};
    use crate::ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use crate::ergotree_ir::chain::ergo_box::{ErgoBox, NonMandatoryRegisters};
    use crate::ergotree_ir::ergo_tree::ErgoTree;
    use std::convert::{TryFrom, TryInto};
    use crate::ergotree_interpreter::sigma_protocol::private_input::{DlogProverInput, PrivateInput};
    use crate::ergotree_ir::chain::base16_bytes::Base16DecodedBytes;
    use crate::ergotree_ir::serialization::SigmaSerializable;
    use crate::ergotree_ir::sigma_protocol::dlog_group;
    use crate::ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
    use k256::Scalar;
    use ergotree_ir::chain::block_id::BlockId;
    use ergotree_ir::chain::json::ergo_tree::serialize;
    use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
    use crate::chain::json::hints::OwnCommitmentJson;
    use crate::chain::json::transaction::TransactionJson;
    use crate::chain::transaction::prover_result::ProverResult;
    use crate::chain::transaction::unsigned::UnsignedTransaction;
    use crate::ergotree_interpreter::sigma_protocol::verifier::{TestVerifier, Verifier};
    use crate::ergotree_ir::chain::digest32::Digest32;
    use crate::ergotree_ir::chain::header::Header;
    use crate::ergotree_ir::chain::preheader::PreHeader;
    use crate::ergotree_ir::chain::votes::Votes;
    use crate::ergotree_ir::mir::expr::Expr;
    use crate::ergotree_ir::mir::sigma_and::SigmaAnd;
    use crate::wallet::signing::sign_transaction_multi;

    #[test]
    fn extract_hint(){
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
        let value_m: BoxValue = BoxValue::new(4400000).unwrap();
        let bytes_m = Base16DecodedBytes::try_from( "100208cd03c847c306a2f9a8087b4ae63261cc5acea9034000ba8d033b0fb033247e8aade908cd02f4b05f44eb9703db7fcf9c94b89566787a7188c7e48964821d485d9ef2f9e4c4ea0273007301").unwrap();
        let tree_m: ErgoTree = ErgoTree::sigma_parse_bytes(&bytes_m.0).unwrap();
        let txid_m: TxId = TxId::zero();
        let input_m: ErgoBox = ErgoBox::new(value_m, tree_m.clone(), None, NonMandatoryRegisters::empty(), 0, txid_m, 0).unwrap();
        let mut inputvec_m = Vec::new();
        inputvec_m.push(input_m);
        let contx = Rc::new(force_any_val::<Context>());
        let exp=tree_m.proposition().unwrap();
        let reduction_result=reduce_to_crypto(&exp,&Env::empty(),contx).unwrap();
        let sigma_tree=reduction_result.sigma_prop;
        let stx:Transaction=serde_json::from_str(signed_tx).unwrap();
        let test:ProofBytes=stx.inputs.first().clone().spending_proof.proof;
        let proof:Vec<u8>=Vec::from(test);
        let mut real_proposition:Vec<SigmaBoolean>=Vec::new();
        let mut simulated_proposition:Vec<SigmaBoolean>=Vec::new();
        let mut bag:HintsBag=bag_for_multi_sig(sigma_tree.clone(),&real_proposition,&simulated_proposition,&proof);
        assert_eq!(bag.hints.is_empty(),true);

        let address_encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let firstaddress: Address = address_encoder.parse_address_from_str("9hz1anoLGZGf88hjjrQ3y3oSpSE2Kkk15CcfFqCNYXDPWKV6F4i").unwrap();
        let secondaddress = address_encoder.parse_address_from_str("9gNpmNsivNnAKb7EtGVLGF24wRUJWnEqycCnWCeYxwSFZxkKyTZ").unwrap();

        match firstaddress {
            Address::P2Pk(pk) => {
                real_proposition.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk)));
            }
            _ => {
                println!("nonde of");
            }
        }
        match secondaddress {
            Address::P2Pk(pk) => {
                simulated_proposition.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk)));
            }
            _ => {
                println!("nonde of");
            }
        }
        bag=bag_for_multi_sig(sigma_tree,&real_proposition,&simulated_proposition,&proof);
        assert_eq!(bag.hints.is_empty(),false);

    }

    #[test]
    fn generating_commitment() {
        let secret1 = DlogProverInput::random();
        let secret2 = DlogProverInput::random();
        let secret3 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let pk3 = secret3.public_image();
        let mut generate_for: Vec<SigmaBoolean> = Vec::new();
        generate_for.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk2.clone())));
        assert_eq!(generate_commitments_for(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())), &generate_for).hints.len(), 0);
        generate_for.clear();
        let cand = Cand::normalized(vec![pk1.clone().into(), pk2.clone().into()].try_into().unwrap());
        generate_for.push(cand.clone());
        assert_eq!(generate_commitments_for(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())), &generate_for).hints.is_empty(), true);
        assert_eq!(generate_commitments_for(cand.clone(), &generate_for).hints.is_empty(), true);
        generate_for.clear();
        generate_for.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())));
        assert_eq!(generate_commitments_for(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())), &generate_for).hints.is_empty(), false);
        generate_for.clear();
        generate_for.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())));
        let mut bag=generate_commitments_for(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())), &generate_for);
        assert_eq!(bag.hints.is_empty(),false);
        let mut hint=bag.hints[0].clone();
        let mut a:Option<FirstProverMessage>=None;
        let mut r:Option<Scalar>=None;
        if let Hint::CommitmentHint(CommitmentHint::RealCommitment(comm)) = hint {
            assert_eq!(comm.position,NodePosition::crypto_tree_prefix().clone());
            a=Some(comm.commitment);

        }
        hint=bag.hints[1].clone();
        if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(comm)) = hint {
            assert_eq!(comm.position,NodePosition::crypto_tree_prefix().clone());
            r=Some(comm.secret_randomness);
        }
        let g_to_r = dlog_group::exponentiate(&dlog_group::generator(), &r.unwrap());
        assert_eq!(FirstProverMessage::FirstDlogProverMessage(g_to_r.into()),a.clone().unwrap());

        bag=generate_commitments_for(cand.clone(), &generate_for);
        assert_eq!(bag.hints.len(), 2);
        hint=bag.hints[0].clone();
        if let Hint::CommitmentHint(CommitmentHint::RealCommitment(comm)) = hint {
            assert_eq!(comm.position,NodePosition{positions:vec![0,0]});
        }
        hint=bag.hints[1].clone();
        if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(comm)) = hint {
            assert_eq!(comm.position,NodePosition{positions:vec![0,0]});
        }


    }

    #[test]
    fn multi(){
        let ctx = Rc::new(force_any_val::<Context>());

        let secret1 = DlogProverInput::random();
        let secret2 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let prover1= TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret1)],
        };
        let prover2= TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret2)],
        };
        let expr: Expr = SigmaAnd::new(vec![Expr::Const(pk1.clone().into()), Expr::Const(pk2.clone().into())])
            .unwrap()
            .into();
        let tree_and=ErgoTree::try_from(expr.clone()).unwrap();

        let reduced_and=tree_and.proposition().unwrap();
        let cand=reduce_to_crypto(&expr,&Env::empty(),ctx.clone()).unwrap().sigma_prop;
        let mut generate_for: Vec<SigmaBoolean> = Vec::new();
        generate_for.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk2.clone())));
        let hints_from_bob:HintsBag=generate_commitments_for(cand.clone(),&generate_for);
        let bag1=hints_from_bob.real_commitments().clone();
        let own=hints_from_bob.own_commitments();
        let test:OwnCommitment=own.get(0).unwrap().clone();
        let message = vec![0u8; 100];


        let mut bag_a =HintsBag{hints:vec![]};
        bag_a.add_hint(Hint::CommitmentHint(CommitmentHint::RealCommitment(bag1.get(0).unwrap().clone())));


        let proof1=prover1.prove(&tree_and,&Env::empty(),ctx.clone(),message.as_slice(),&bag_a).unwrap();
        let proof:Vec<u8>=Vec::from(proof1.proof.clone());
        let mut real_proposition:Vec<SigmaBoolean>=Vec::new();
        let mut simulated_proposition:Vec<SigmaBoolean>=Vec::new();
        real_proposition.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())));
        let mut bag_b=bag_for_multi_sig(cand.clone(),&real_proposition,&simulated_proposition,&proof);
        bag_b.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(own.get(0).unwrap().clone())));
        let proof2=prover2.prove(&tree_and,&Env::empty(),ctx.clone(),message.as_slice(),&bag_b).unwrap();
        let proof_byte:ProofBytes=(proof2.proof.clone());
        let verifier = TestVerifier;
        let test:Vec<u8>=Vec::from(proof2.proof.clone());


        assert_eq!(verifier.verify(&tree_and,
                        &Env::empty(),
                        ctx,
                        proof_byte.clone(),
                        message.as_slice())
            .unwrap().result,
        true);

        assert_eq!(1,1);



    }

    #[test]
    fn multi_sig(){
        let unsigend=r#"
           {
          "inputs": [
            {
              "boxId": "bd85fe13b44d9e928a784f850c911d8b754196b9b8331255e79caf51040428d3",
              "extension": {}
            }
          ],
          "dataInputs": [],
          "outputs": [
            {
              "value": 42900000,
              "ergoTree": "100208cd02a65551523e09530bc8de55d426ff60aace7388cef08f5477c8726713f68c0e7808cd039c8404d33f85dd4012e4f3d0719951eeea0015b13b940d67d4990e13de28b154ea0273007301",
              "assets": null,
              "additionalRegisters": {},
              "creationHeight": 0
            },
            {
              "value": 1100000,
              "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
              "assets": null,
              "additionalRegisters": {},
              "creationHeight": 0
            }
          ]
        }
    "#;
        let tx: UnsignedTransaction = serde_json::from_str(unsigend).unwrap();
        let value: BoxValue = BoxValue::new(44000000).unwrap();
        let bytes = Base16DecodedBytes::try_from("100208cd02a65551523e09530bc8de55d426ff60aace7388cef08f5477c8726713f68c0e7808cd039c8404d33f85dd4012e4f3d0719951eeea0015b13b940d67d4990e13de28b154ea0273007301").unwrap();
        let tree: ErgoTree = ErgoTree::sigma_parse_bytes(&bytes.0).unwrap();
        let decoded = hex::decode("7c340026267640f32d58c64c92c949428505ddce3f89c120614fbe4bd007fca9").expect("Decoding failed");
        let txiddigest:Digest32=Digest32::from(<[u8; 32]>::try_from(decoded.as_slice()).unwrap());
        let txid:TxId=TxId(txiddigest);
        let input: ErgoBox = ErgoBox::new(value, tree.clone(), None, NonMandatoryRegisters::empty(), 79766, txid, 0).unwrap();
        let mut inputvec = Vec::new();
        inputvec.push(input);
        let input_boxes: TxIoVec<ErgoBox> = TxIoVec::from_vec(inputvec).unwrap();
        let transactionctx: TransactionContext = TransactionContext::new(tx, input_boxes, None).unwrap();
        let propo = tree.proposition().unwrap();

        let parent_decoded = hex::decode("c77bdf0705ab118d77cebcdc98a3e8f673b3f4d08fd017e14f194fe85f91bb3b").expect("Decoding failed");
        let parentid:Digest32=Digest32::from(<[u8; 32]>::try_from(parent_decoded.as_slice()).unwrap());

        let minerpk=EcPoint::from_base16_str("02c968d8d8c6eafb8f4682c6959a7b2073ae2dfae02eb20a3be82e4b497d5e6c53".to_string()).unwrap();
        let pre_header:PreHeader=PreHeader{
            version:2,
            parent_id:BlockId(parentid),
            timestamp:1636835521142,
            n_bits:83973853,
            height:80084,
            miner_pk:minerpk.into(),
            votes:Votes([0u8; 3]),
        };



        let ctx:ErgoStateContext=ErgoStateContext{pre_header,headers:vec![Header::dummy(); 10].try_into().expect("error")};


        // let ctx: ErgoStateContext = ErgoStateContext::dummy();
        let reduced_tx: ReducedTransaction = reduce_tx(transactionctx.clone(), &ctx).unwrap();
        let secret1:DlogProverInput=DlogProverInput::from_base16_str("49cca0bf75664a8cd35ade351486d4e926744552aafff4bc797ccce968f4d78a".to_string()).unwrap();
        let secret2:DlogProverInput=DlogProverInput::from_base16_str("77d670184af0b5f0f257f98afc8067f572283f28976d1413bfe781fe7b95e949".to_string()).unwrap();
        let addressencoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let address1: Address = addressencoder.parse_address_from_str("9fnKGv65AXT2AKahaRjczQ4eQvJbiiyNumPye3SXxxs2MZS7iRE").unwrap();
        let address2:Address = addressencoder.parse_address_from_str("9hejg6bAkWWDVThFCdr47ATUXkiwhttDdZpSGHn6Zkn8BUoEP1t").unwrap();
        let pk1=secret1.public_image();
        let pk2=secret2.public_image();
        let prover1= TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret1)],
        };
        let prover2= TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret2)],
        };
        let vecreduced_input: TxIoVec<ReducedInput> = reduced_tx.reduced_inputs();
        let reducedinput: ReducedInput = (vecreduced_input.first()).clone();
        let reductionresult: ReductionResult = reducedinput.reduction_result;
        let sigmaprop: SigmaBoolean = reductionresult.sigma_prop;

        let mut generate_for: Vec<SigmaBoolean> = Vec::new();
        let bag = generate_commitments_for(sigmaprop.clone(), &generate_for);
        generate_for.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk2.clone())));
        let hints_from_bob:HintsBag=generate_commitments_for(sigmaprop.clone(),&generate_for);
        let decoded = hex::decode("7c340026267640f32d58c64c92c949428505ddce3f89c120614fbe4bd007fca9").expect("Decoding failed");
        let txiddigest:Digest32=Digest32::from(<[u8; 32]>::try_from(decoded.as_slice()).unwrap());

        let bag1=hints_from_bob.real_commitments().clone();
        let own=hints_from_bob.own_commitments();

        // let message = vec![0u8; 100];
        let tx1 = transactionctx.spending_tx.clone();
        let message= tx1.bytes_to_sign().unwrap();
        let mut bag_a =HintsBag{hints:vec![]};
        bag_a.add_hint(Hint::CommitmentHint(CommitmentHint::RealCommitment(bag1.get(0).unwrap().clone())));

        // let res = sign_transaction_multi(prover1.as_ref(), tx_context.clone(), &force_any_val::<ErgoStateContext>());
        // let signed_tx = res.unwrap();
        let ctx1 = Rc::new(force_any_val::<Context>());
        let mut transaction_bag1:TransactionHintsBag=TransactionHintsBag::empty();
        transaction_bag1.add_hints_for_input(0,bag_a);
        let test=sign_transaction_multi(&prover1,transaction_bag1,transactionctx.clone(),&ctx).unwrap();
        let proof1=test.inputs.get(0).unwrap().spending_proof.clone();

        // let proof1=prover1.prove(&tree,&Env::empty(),ctx1.clone(),message.as_slice(),&bag_a).unwrap();
        let proof:Vec<u8>=Vec::from(proof1.proof.clone());

        let mut real_proposition:Vec<SigmaBoolean>=Vec::new();
        let mut simulated_proposition:Vec<SigmaBoolean>=Vec::new();
        real_proposition.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())));
        let mut bag_b=bag_for_multi_sig(sigmaprop.clone(),&real_proposition,&simulated_proposition,&proof);

        bag_b.add_hint(Hint::CommitmentHint(CommitmentHint::OwnCommitment(own.get(0).unwrap().clone())));
        println!("bag size is {}", bag_b.hints.len());
        let mut transaction_bag:TransactionHintsBag=TransactionHintsBag::empty();
        transaction_bag.add_hints_for_input(0,bag_b);
        let test=sign_transaction_multi(&prover2,transaction_bag,transactionctx.clone(),&ctx).unwrap();

        // let proof2=prover2.prove(&tree,&Env::empty(),ctx1.clone(),message.as_slice(),&bag_b).unwrap();
        // let proof_byte:ProofBytes=(proof2.proof.clone());
        // println!("{}",String::from(proof2.proof.clone()));
        // let teesst:TransactionJson=TransactionJson::from(test.clone());
        // println!("{}",serde_json::to_string(&test).unwrap());

        let verifier = TestVerifier;
        let proof_byte:ProofBytes=(test.inputs.get(0).unwrap().spending_proof.clone().proof.clone());


        assert_eq!(verifier.verify(&tree,
                                   &Env::empty(),
                                   ctx1,
                                   proof_byte.clone(),
                                   message.as_slice())
                       .unwrap().result,
                   true);


        // pub fn sign_transaction_multi(
        //     prover: &dyn Prover,
        //     tx_hints: TransactionHintsBag,
        //     tx_context: TransactionContext,
        //     state_context: &ErgoStateContext,
        // )


            let (r, a) = interactive_prover::first_message();
            let owncmt=

                    OwnCommitment {
                        image: SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
                        secret_randomness: r,
                        commitment: FirstProverMessage::FirstDlogProverMessage(
                            a.clone()
                        ),
                        position: NodePosition::crypto_tree_prefix().clone(),
                    }
                ;
        let json:OwnCommitmentJson=OwnCommitmentJson::from(owncmt);
        println!("{}",serde_json::to_string(&json).unwrap());

        // let reverse=r#"{"secret":"fe9f7dd9f194a7ff258b3be53fb73531a238f7a4aa45975b415647b0e4333f40","position":"0","a":"02e5361c9ff16077285322b231503401386212f28c19d09c6975ec6ab991035c77","image":"02a65551523e09530bc8de55d426ff60aace7388cef08f5477c8726713f68c0e78"}"#;
        let reverse=serde_json::to_string(&json).unwrap();
        let own_com_json:OwnCommitmentJson=serde_json::from_str(&reverse).unwrap();
        let own_com:OwnCommitment=OwnCommitment::from(own_com_json);
        println!("{}",hex::encode(own_com.secret_randomness.clone().to_bytes().as_slice()));


    }

}