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

#[cfg(test)]
mod tests{
    use std::rc::Rc;
    use super::*;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;
    use crate::ergotree_interpreter::eval::context::Context;

    #[test]

    fn extract_hint(){
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(1,1);
    }


}
