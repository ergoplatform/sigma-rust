//! Unchecked proof tree types

use ergo_chain_types::Base16EncodedBytes;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use gf2_192::gf2_192poly::Gf2_192Poly;

use super::dht_protocol::FirstDhTupleProverMessage;
use super::dht_protocol::SecondDhTupleProverMessage;
use super::proof_tree::ConjectureType;
use super::proof_tree::ProofTree;
use super::proof_tree::ProofTreeConjecture;
use super::proof_tree::ProofTreeKind;
use super::proof_tree::ProofTreeLeaf;
use super::sig_serializer::serialize_sig;
use super::{
    dlog_protocol::{FirstDlogProverMessage, SecondDlogProverMessage},
    Challenge, FirstProverMessage,
};

use derive_more::From;

/// Unchecked sigma tree
#[derive(PartialEq, Debug, Clone, From)]
#[cfg_attr(
    feature = "json",
    derive(serde::Serialize),
    serde(into = "ergo_chain_types::Base16EncodedBytes")
)]
pub enum UncheckedTree {
    /// Unchecked leaf
    UncheckedLeaf(UncheckedLeaf),
    /// Unchecked conjecture (OR, AND, ...)
    UncheckedConjecture(UncheckedConjecture),
}

impl UncheckedTree {
    /// Get challenge
    pub(crate) fn challenge(&self) -> Challenge {
        match self {
            UncheckedTree::UncheckedLeaf(ul) => ul.challenge(),
            UncheckedTree::UncheckedConjecture(uc) => uc.challenge(),
        }
    }

    pub(crate) fn as_tree_kind(&self) -> ProofTreeKind {
        match self {
            UncheckedTree::UncheckedLeaf(ul) => ProofTreeKind::Leaf(ul),
            UncheckedTree::UncheckedConjecture(uc) => ProofTreeKind::Conjecture(uc),
        }
    }

    pub(crate) fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UncheckedTree::UncheckedLeaf(ul) => ul.with_challenge(challenge).into(),
            UncheckedTree::UncheckedConjecture(uc) => uc.with_challenge(challenge).into(),
        }
    }
}

impl From<UncheckedTree> for Base16EncodedBytes {
    fn from(tree: UncheckedTree) -> Self {
        let bytes: Vec<u8> = serialize_sig(tree).into();
        Base16EncodedBytes::from(bytes)
    }
}

/// Unchecked leaf
#[derive(PartialEq, Debug, Clone, From)]
pub enum UncheckedLeaf {
    /// Unchecked Schnorr
    UncheckedSchnorr(UncheckedSchnorr),
    /// Unchecked DhTuple
    UncheckedDhTuple(UncheckedDhTuple),
}

impl UncheckedLeaf {
    /// Challenge of FiatShamir
    pub fn challenge(&self) -> Challenge {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => us.challenge.clone(),
            UncheckedLeaf::UncheckedDhTuple(udht) => udht.challenge.clone(),
        }
    }
    /// Set Challenge
    pub fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => us.with_challenge(challenge).into(),
            UncheckedLeaf::UncheckedDhTuple(udht) => udht.with_challenge(challenge).into(),
        }
    }
}

impl ProofTreeLeaf for UncheckedLeaf {
    fn proposition(&self) -> SigmaBoolean {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(us.proposition.clone()),
            ),
            UncheckedLeaf::UncheckedDhTuple(dhu) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDhTuple(dhu.proposition.clone()),
            ),
        }
    }
    fn commitment_opt(&self) -> Option<FirstProverMessage> {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => us.commitment_opt.clone().map(Into::into),
            UncheckedLeaf::UncheckedDhTuple(udh) => udh.commitment_opt.clone().map(Into::into),
        }
    }
}
/// Unchecked Schnorr
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct UncheckedSchnorr {
    /// Proposition
    pub proposition: ProveDlog,
    /// Commitment FirstDlogProverMessage
    pub commitment_opt: Option<FirstDlogProverMessage>,
    /// Challenge
    pub challenge: Challenge,
    /// SecondMessage
    pub second_message: SecondDlogProverMessage,
}

impl UncheckedSchnorr {
    /// Set New Challenge
    pub fn with_challenge(self, challenge: Challenge) -> Self {
        UncheckedSchnorr { challenge, ..self }
    }
}

impl From<UncheckedSchnorr> for UncheckedTree {
    fn from(us: UncheckedSchnorr) -> Self {
        UncheckedTree::UncheckedLeaf(us.into())
    }
}

impl From<UncheckedDhTuple> for UncheckedTree {
    fn from(dh: UncheckedDhTuple) -> Self {
        UncheckedTree::UncheckedLeaf(dh.into())
    }
}

/// UncheckedDhTuple
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct UncheckedDhTuple {
    /// Proposition
    pub proposition: ProveDhTuple,
    /// Commitment  FirstDhTupleProverMessage
    pub commitment_opt: Option<FirstDhTupleProverMessage>,
    /// Challenge
    pub challenge: Challenge,
    /// SecondMessage
    pub second_message: SecondDhTupleProverMessage,
}

impl UncheckedDhTuple {
    /// Set Challenge
    pub fn with_challenge(self, challenge: Challenge) -> Self {
        UncheckedDhTuple { challenge, ..self }
    }
}

/// UncheckedConjecture
#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum UncheckedConjecture {
    /// Unchecked And Conjecture
    CandUnchecked {
        /// Challenge
        challenge: Challenge,
        /// Children
        children: SigmaConjectureItems<UncheckedTree>,
    },
    /// Unchecked Or Conjecture
    CorUnchecked {
        /// Challenge
        challenge: Challenge,
        /// Children
        children: SigmaConjectureItems<UncheckedTree>,
    },
    /// Unchecked Cthreshold Conjecture
    CthresholdUnchecked {
        /// Challenge
        challenge: Challenge,
        /// Children
        children: SigmaConjectureItems<UncheckedTree>,
        /// K
        k: u8,
        /// Polynomial
        polynomial: Gf2_192Poly,
    },
}

impl UncheckedConjecture {
    /// Set New Children
    pub fn with_children(self, new_children: SigmaConjectureItems<UncheckedTree>) -> Self {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge,
                children: _,
            } => UncheckedConjecture::CandUnchecked {
                challenge,
                children: new_children,
            },
            UncheckedConjecture::CorUnchecked {
                challenge,
                children: _,
            } => UncheckedConjecture::CorUnchecked {
                challenge,
                children: new_children,
            },
            UncheckedConjecture::CthresholdUnchecked {
                challenge,
                children: _,
                k,
                polynomial: polynomial_opt,
            } => UncheckedConjecture::CthresholdUnchecked {
                challenge,
                children: new_children,
                k,
                polynomial: polynomial_opt,
            },
        }
    }
    /// Get Children
    pub fn children_ust(self) -> SigmaConjectureItems<UncheckedTree> {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge: _,
                children,
            } => children,
            UncheckedConjecture::CorUnchecked {
                challenge: _,
                children,
            } => children,
            UncheckedConjecture::CthresholdUnchecked {
                challenge: _,
                children,
                k: _,
                polynomial: _,
            } => children,
        }
    }
    /// Get Children
    pub fn challenge(&self) -> Challenge {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge,
                children: _,
            } => challenge.clone(),
            UncheckedConjecture::CorUnchecked {
                challenge,
                children: _,
            } => challenge.clone(),
            UncheckedConjecture::CthresholdUnchecked {
                challenge,
                children: _,
                k: _,
                polynomial: _,
            } => challenge.clone(),
        }
    }
    /// Set Challenge
    pub fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge: _,
                children,
            } => UncheckedConjecture::CandUnchecked {
                challenge,
                children,
            },
            UncheckedConjecture::CorUnchecked {
                challenge: _,
                children,
            } => UncheckedConjecture::CorUnchecked {
                challenge,
                children,
            },
            UncheckedConjecture::CthresholdUnchecked {
                challenge: _,
                children,
                k,
                polynomial: polynomial_opt,
            } => UncheckedConjecture::CthresholdUnchecked {
                challenge,
                children,
                k,
                polynomial: polynomial_opt,
            },
        }
    }
}

impl ProofTreeConjecture for UncheckedConjecture {
    /// Get Conjecture Type
    fn conjecture_type(&self) -> ConjectureType {
        match self {
            UncheckedConjecture::CandUnchecked { .. } => ConjectureType::And,
            UncheckedConjecture::CorUnchecked { .. } => ConjectureType::Or,
            UncheckedConjecture::CthresholdUnchecked { .. } => ConjectureType::Threshold,
        }
    }

    /// Get Children
    fn children(&self) -> SigmaConjectureItems<ProofTree> {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge: _,
                children,
            } => children.mapped_ref(|ust| ust.clone().into()),
            UncheckedConjecture::CorUnchecked {
                challenge: _,
                children,
            } => children.mapped_ref(|ust| ust.clone().into()),
            UncheckedConjecture::CthresholdUnchecked {
                challenge: _,
                children,
                k: _,
                polynomial: _,
            } => children.mapped_ref(|ust| ust.clone().into()),
        }
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use std::convert::TryInto;

    use crate::sigma_protocol::gf2_192::gf2_192poly_from_byte_array;

    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    pub fn primitive_type_value() -> BoxedStrategy<UncheckedTree> {
        prop_oneof![
            any::<UncheckedSchnorr>().prop_map_into(),
            any::<UncheckedDhTuple>().prop_map_into(),
        ]
        .boxed()
    }

    impl Arbitrary for UncheckedTree {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            primitive_type_value()
                .prop_recursive(1, 6, 4, |elem| {
                    prop_oneof![
                        (vec(elem.clone(), 2..=3), any::<Challenge>())
                            .prop_map(|(elems, challenge)| UncheckedConjecture::CandUnchecked {
                                children: elems.try_into().unwrap(),
                                challenge,
                            })
                            .prop_map_into(),
                        (vec(elem.clone(), 2..=3), any::<Challenge>())
                            .prop_map(|(elems, challenge)| UncheckedConjecture::CorUnchecked {
                                children: elems.try_into().unwrap(),
                                challenge
                            })
                            .prop_map_into(),
                        (vec(elem, 2..=4), any::<Challenge>(), vec(any::<u8>(), 24))
                            .prop_map(|(elems, challenge, random_bytes_for_one_absent_signer)| {
                                let polynomial = gf2_192poly_from_byte_array(
                                    challenge.clone(),
                                    random_bytes_for_one_absent_signer,
                                )
                                .unwrap();
                                UncheckedConjecture::CthresholdUnchecked {
                                    k: (elems.len() - 1) as u8,
                                    children: elems.try_into().unwrap(),
                                    challenge,
                                    polynomial,
                                }
                            })
                            .prop_map_into(),
                    ]
                })
                .boxed()
        }
    }
}
