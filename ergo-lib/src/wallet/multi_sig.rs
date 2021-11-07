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
use crate::chain::transaction::TxIoVec;
use crate::wallet::signing::TransactionContext;
use crate::chain::ergo_state_context::ErgoStateContext;
use ergotree_interpreter::sigma_protocol::unchecked_tree::{UncheckedConjecture, UncheckedLeaf, UncheckedTree};
use ergotree_interpreter::sigma_protocol::dlog_protocol::{interactive_prover::compute_commitment};
use crate::ergotree_interpreter::sigma_protocol::proof_tree::ProofTreeLeaf;
use ergotree_interpreter::sigma_protocol::sig_serializer::parse_sig_compute_challenges;

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

pub fn traverse_node(
    tree:UncheckedTree,
    real_propositions: &Vec<SigmaBoolean>,
    simulated_propositions:&Vec<SigmaBoolean>,
    position:NodePosition,
    bag: &mut HintsBag
){
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
            if real_found||simulated_found {
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
    use crate::ergotree_interpreter::eval::reduce_to_crypto;
    use crate::ergotree_interpreter::sigma_protocol::prover::ProofBytes;
    use crate::ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use crate::ergotree_ir::chain::ergo_box::{ErgoBox, NonMandatoryRegisters};
    use crate::ergotree_ir::ergo_tree::ErgoTree;
    use std::convert::{TryFrom, TryInto};
    use crate::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
    use crate::ergotree_ir::chain::base16_bytes::Base16DecodedBytes;
    use crate::ergotree_ir::serialization::SigmaSerializable;
    use crate::ergotree_ir::sigma_protocol::dlog_group;
    use crate::ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
    use k256::Scalar;

    #[test]
    fn extract_hint(){
        let signed_tx = r#"{
          "id": "bb1a12f931d658324719000d6f87036f366d7279dbb43988c7857bbce8d4925b",
          "inputs": [
            {
              "boxId": "b43d88a35110167465a3653934c8aefd06bfb6dff5b6a9bf8d278fbd2780c28a",
              "spendingProof": {
                "proofBytes": "2de70a8706cd032fa26db226eb57cb099c53f7ce945590d31f1f341bb52b743ab19429c896c68a1d2944d67462ca917f4c65760740fff229563c8a49825c94222eaaeebacaa9627682364406f06bf13d4a2f325792830b70",
                "extension": {}
              }
            }
          ],
          "dataInputs": [],
          "outputs": [
            {
              "boxId": "a5ab41b243eb9a20236cc0135b0bef05c3e9f3917c4ed707b4343dd0043cd85b",
              "value": 1000000,
              "ergoTree": "0008cd039c8404d33f85dd4012e4f3d0719951eeea0015b13b940d67d4990e13de28b154",
              "assets": [],
              "additionalRegisters": {},
              "creationHeight": 0,
              "transactionId": "bb1a12f931d658324719000d6f87036f366d7279dbb43988c7857bbce8d4925b",
              "index": 0
            },
            {
              "boxId": "206bcbee98db66b75d0ad83c522e8f2ea1809f13955d195ba1fb77feacd9aaf6",
              "value": 2300000,
              "ergoTree": "0008cd039c8404d33f85dd4012e4f3d0719951eeea0015b13b940d67d4990e13de28b154",
              "assets": [],
              "additionalRegisters": {},
              "creationHeight": 0,
              "transactionId": "bb1a12f931d658324719000d6f87036f366d7279dbb43988c7857bbce8d4925b",
              "index": 1
            },
            {
              "boxId": "43c86e260abfa05f2f4a9daea9dccebd0e73b64a7dd067401feb134cca0a535e",
              "value": 1100000,
              "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
              "assets": [],
              "additionalRegisters": {},
              "creationHeight": 0,
              "transactionId": "bb1a12f931d658324719000d6f87036f366d7279dbb43988c7857bbce8d4925b",
              "index": 2
            }
          ]
        }"#;
        let value_m: BoxValue = BoxValue::new(4400000).unwrap();
        let bytes_m = Base16DecodedBytes::try_from("100208cd021041ea459227841f3cc0886989f485e7fca5254421040ec91ab3494\
        c498eacf808cd0366d190a37d849d76735f860a2eabb3ba02202aa2a211566d869e848ab0d01154ea0273007301").unwrap();
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
        let real_proposition:Vec<SigmaBoolean>=Vec::new();
        let simulated_proposition:Vec<SigmaBoolean>=Vec::new();
        let ans:HintsBag=bag_for_multi_sig(sigma_tree,&real_proposition,&simulated_proposition,&proof);
        assert_eq!(ans.hints.is_empty(),true);
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

}
