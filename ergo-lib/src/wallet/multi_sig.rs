//! multi sig prover

use crate::chain::transaction::reduced::{reduce_tx, ReducedTransaction, ReducedInput};
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::{SigmaProofOfKnowledgeTree, ProveDlog};
use crate::ergotree_interpreter::sigma_protocol::dlog_protocol::interactive_prover;
use crate::ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
use crate::ergotree_interpreter::sigma_protocol::prover::hint::{OwnCommitment,
                                                                RealCommitment,
                                                                HintsBag,
                                                                Hint,
                                                                CommitmentHint};
use crate::ergotree_interpreter::sigma_protocol::FirstProverMessage;
use k256::Scalar;
use crate::ergotree_ir::chain::address::{AddressEncoder, Address};
use crate::ergotree_ir::chain::address::NetworkPrefix;
use std::collections::HashMap;
use crate::chain::transaction::TxIoVec;
use crate::wallet::signing::TransactionContext;
use crate::chain::ergo_state_context::ErgoStateContext;

/// TransactionHintsBag
pub struct TransactionHintsBag {
    secret_hints: HashMap<usize, HintsBag>,
    public_hints: HashMap<usize, HintsBag>,
}

/// implementation for TransactionHintsBag
impl TransactionHintsBag {
    pub fn replace_hints_for_input(&mut self, index: usize, hints_bag: HintsBag) {
        let public: Vec<Hint> = hints_bag.hints.clone().into_iter().filter(|hint| {
            if let Hint::CommitmentHint(CommitmentHint::RealCommitment(_)) = hint {
                true
            } else {
                false
            }
        }).collect();
        let secret: Vec<Hint> = hints_bag.hints.clone().into_iter().filter(|hint| {
            if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(_)) = hint {
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
            if let Hint::CommitmentHint(CommitmentHint::RealCommitment(_)) = hint {
                true
            } else {
                false
            }
        }).collect();
        let mut secret: Vec<Hint> = hints_bag.hints.clone().into_iter().filter(|hint| {
            if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(_)) = hint {
                true
            } else {
                false
            }
        }).collect();
        let mut secret_bag = HintsBag::empty();
        let mut public_bag = HintsBag::empty();
        let mut old_secret: &Vec<Hint> = &self.secret_hints.get(&index)
            .unwrap_or(&secret_bag).hints;
        for hint in old_secret {
            secret.push(hint.clone());
        }

        let mut old_public: &Vec<Hint> = &self.public_hints.get(&index)
            .unwrap_or(&public_bag).hints;
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
        let mut secrets: &Vec<Hint> = &self.secret_hints.get(&index)
            .unwrap_or(&secret_bag).hints;
        for hint in secrets {
            hints.push(hint.clone());
        }
        let mut public: &Vec<Hint> = &self.public_hints.get(&index)
            .unwrap_or(&public_bag).hints;
        for hint in public {
            hints.push(hint.clone());
        }
        let mut hints_bag: HintsBag = HintsBag { hints };
        return hints_bag;
    }
}

pub fn generate_commitments(
    tx_context: TransactionContext,
    state_context: &ErgoStateContext,
) -> TransactionHintsBag {
    let reduced_tx: ReducedTransaction = reduce_tx(tx_context, state_context).unwrap();
    let reduced_input: TxIoVec<ReducedInput> = reduced_tx.reduced_inputs();
    let mut hints_bag: TransactionHintsBag = TransactionHintsBag {
        secret_hints: HashMap::new(),
        public_hints: HashMap::new(),
    };

    reduced_input.iter().enumerate().for_each(|(i, input)| {
        let sigma_prop: SigmaBoolean = input.clone().reduction_result.sigma_prop;
        hints_bag.add_hints_for_input(i, generateCommitments(sigma_prop));
    });
    return hints_bag;
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
                        SigmaProofOfKnowledgeTree::ProveDlog(_) => {
                            let (r, a) = interactive_prover::
                            first_message();
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