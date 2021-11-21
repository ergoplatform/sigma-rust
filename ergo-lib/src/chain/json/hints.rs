//! Hints JSON wrapper

use std::convert::TryFrom;
use k256::{Scalar};
use serde::{Deserialize, Serialize};
use crate::ergotree_interpreter::sigma_protocol::prover::hint::{CommitmentHint, OwnCommitment, SimulatedCommitment};
use ergotree_interpreter::sigma_protocol::prover::hint::RealCommitment;
use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::ergotree_interpreter::sigma_protocol::{FirstProverMessage, ProverMessage};
use crate::ergotree_ir::serialization::SigmaSerializable;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use ergotree_interpreter::sigma_protocol::dlog_protocol::FirstDlogProverMessage;
use ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog as OtherProveDlog;

/// OwnCommitmentJson serialization
#[derive(Serialize, Deserialize)]
pub struct OwnCommitmentJson {
    /// secret
    pub secret: String,
    /// position
    pub position: String,
    /// a
    pub a: String,
    /// image
    pub image: String,
}

/// RealCommitmentJson serialization
#[derive(Serialize, Deserialize)]
pub struct RealCommitmentJson {
    /// image
    pub image: String,
    /// position
    pub position: String,
    /// a
    pub a: String,
}

/// SimulatedCommitmentJson serialization
#[derive(Serialize, Deserialize)]
pub struct SimulatedCommitmentJson {
    /// image
    pub image: String,
    /// position
    pub position: String,
    /// a
    pub a: String,
}

/// PublicKeyJson serialization
#[derive(Serialize, Deserialize, Clone)]
pub struct PublicKeyJson {
    /// op
    pub op: i32,
    /// h
    pub h: String,
}

/// Commitments Hint Json format same as node api
#[derive(Serialize, Deserialize, Clone)]
pub struct CommitmentHintJson {
    /// hint
    pub hint: String,
    /// public key
    pub pubkey: PublicKeyJson,
    /// position
    pub position: String,
    /// proof type
    #[serde(rename = "type")]
    pub proof_type: String,
    /// a
    pub a: String,
    /// secret
    pub secret: Option<String>,
}

impl CommitmentHintJson {

    /// Extracts image(SigmaBoolean)
    pub fn image(&self) -> SigmaBoolean {
        SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(self.pubkey.h.clone()).unwrap())))
    }

    /// Extracts position(NodePosition)
    pub fn position(&self) -> NodePosition {
        let position = str::replace(self.position.clone().as_ref(), "-", "");
        NodePosition { positions: position.chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect() }
    }

    /// Extracts commitment(FirstProverMessage)
    pub fn commitment(&self) -> FirstProverMessage {
        FirstProverMessage::FirstDlogProverMessage(FirstDlogProverMessage::from(EcPoint::from_base16_str(self.a.clone()).unwrap()))
    }
}

// todo trait should be implemented to avoid from duplicated code
impl From<CommitmentHint> for CommitmentHintJson {
    fn from(v: CommitmentHint) -> Self {
        let mut hint: Option<String> = None;
        let mut secret: Option<String> = None;
        let mut a: Option<String> = None;
        let proof_type = "dlog".to_string();
        let mut position: Option<String> = None;
        let mut ec_point: Option<String> = None;
        match v {
            CommitmentHint::OwnCommitment(cmt) => {
                hint = Some("cmtWithSecret".to_string());
                secret = Some(hex::encode(cmt.secret_randomness.clone().to_bytes().as_slice()));
                a = Some(hex::encode(cmt.commitment.clone().bytes().as_slice()));
                position = Some(cmt.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32, 10).unwrap().to_string()).collect::<Vec<_>>().join("-"));
                ec_point = Some(hex::encode(cmt.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string());
            }
            CommitmentHint::RealCommitment(cmt) => {
                hint = Some("cmtReal".to_string());
                a = Some(hex::encode(cmt.commitment.clone().bytes().as_slice()));
                position = Some(cmt.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32, 10).unwrap().to_string()).collect::<Vec<_>>().join("-"));
                ec_point = Some(hex::encode(cmt.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string());
            }
            CommitmentHint::SimulatedCommitment(cmt) => {
                hint = Some("cmtSimulated".to_string());
                a = Some(hex::encode(cmt.commitment.clone().bytes().as_slice()));
                position = Some(cmt.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32, 10).unwrap().to_string()).collect::<Vec<_>>().join("-"));
                ec_point = Some(hex::encode(cmt.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string());
            }
        }
        let public_key = PublicKeyJson {
            op: -51,
            h: ec_point.unwrap(),
        };

        CommitmentHintJson {
            hint: hint.unwrap(),
            pubkey: public_key,
            position: position.unwrap(),
            proof_type,
            a: a.unwrap(),
            secret,
        }
    }
}

impl TryFrom<CommitmentHintJson> for CommitmentHint {
    type Error = &'static str;

    fn try_from(v: CommitmentHintJson) -> Result<CommitmentHint, &'static str> {
        match v.hint.clone().as_ref() {
            "cmtWithSecret" => {
                Ok(
                    CommitmentHint::OwnCommitment(OwnCommitment {
                        secret_randomness: Scalar::from_bytes_reduced(hex::decode(v.secret.clone().unwrap()).unwrap().as_slice().into()),
                        image: v.image(),
                        position: v.position(),
                        commitment: v.commitment(),

                    })
                )
            }
            "cmtReal" => {
                Ok(
                    CommitmentHint::RealCommitment(
                        RealCommitment {
                            image: v.image(),
                            position: v.position(),
                            commitment: v.commitment(),
                        }
                    )
                )
            }
            "cmtSimulated" => {
                Ok(
                    CommitmentHint::SimulatedCommitment(
                        SimulatedCommitment {
                            image: v.image(),
                            position: v.position(),
                            commitment: v.commitment(),
                        }
                    )
                )
            }
            _ => {
                Err("invalid header length")
            }
        }
    }
}

impl From<OwnCommitment> for OwnCommitmentJson {
    fn from(v: OwnCommitment) -> Self {
        let ec_point = &hex::encode(v.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string();

        OwnCommitmentJson {
            secret: hex::encode(v.secret_randomness.clone().to_bytes().as_slice()),
            position: v.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32, 10).unwrap()).collect(),
            a: hex::encode(v.commitment.clone().bytes().as_slice()),
            image: ec_point.clone(),
        }
    }
}

impl From<RealCommitment> for RealCommitmentJson {
    fn from(v: RealCommitment) -> Self {
        let ec_point = &hex::encode(v.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string();
        RealCommitmentJson {
            position: v.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32, 10).unwrap()).collect(),
            a: hex::encode(v.commitment.clone().bytes().as_slice()),
            image: ec_point.clone(),
        }
    }
}

impl From<SimulatedCommitment> for SimulatedCommitmentJson {
    fn from(v: SimulatedCommitment) -> Self {
        let ec_point = &hex::encode(v.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string();
        SimulatedCommitmentJson {
            position: v.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32, 10).unwrap()).collect(),
            a: hex::encode(v.commitment.clone().bytes().as_slice()),
            image: ec_point.clone(),
        }
    }
}

impl From<OwnCommitmentJson> for OwnCommitment {
    fn from(v: OwnCommitmentJson) -> Self {
        OwnCommitment {
            image: SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(v.image.clone()).unwrap()))),
            secret_randomness: Scalar::from_bytes_reduced(hex::decode(v.secret.clone()).unwrap().as_slice().into()),
            position: NodePosition { positions: v.position.clone().chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect() },
            commitment: FirstProverMessage::FirstDlogProverMessage(FirstDlogProverMessage::from(EcPoint::from_base16_str(v.a.clone()).unwrap())),
        }
    }
}

impl From<RealCommitmentJson> for RealCommitment {
    fn from(v: RealCommitmentJson) -> Self {
        RealCommitment {
            image: SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(v.image.clone()).unwrap()))),
            position: NodePosition { positions: v.position.clone().chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect() },
            commitment: FirstProverMessage::FirstDlogProverMessage(FirstDlogProverMessage::from(EcPoint::from_base16_str(v.a.clone()).unwrap())),
        }
    }
}

impl From<SimulatedCommitmentJson> for SimulatedCommitment {
    fn from(v: SimulatedCommitmentJson) -> Self {
        SimulatedCommitment {
            image: SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(v.image.clone()).unwrap()))),
            position: NodePosition { positions: v.position.clone().chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect() },
            commitment: FirstProverMessage::FirstDlogProverMessage(FirstDlogProverMessage::from(EcPoint::from_base16_str(v.a.clone()).unwrap())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use ergotree_interpreter::sigma_protocol::prover::hint::CommitmentHint;
    use crate::chain::json::hints::{CommitmentHintJson, OwnCommitmentJson};
    use crate::ergotree_interpreter::sigma_protocol::dlog_protocol::interactive_prover;
    use crate::ergotree_interpreter::sigma_protocol::FirstProverMessage;
    use crate::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
    use crate::ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
    use crate::ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProofOfKnowledgeTree};
    use crate::ergotree_interpreter::sigma_protocol::prover::hint::{OwnCommitment};

    #[test]
    fn round_trip_own_commitment() {
        let secret1 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let (r, a) = interactive_prover::first_message();
        let own_commitment = OwnCommitment
        {
            image: SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
            secret_randomness: r,
            commitment: FirstProverMessage::FirstDlogProverMessage(
                a.clone()
            ),
            position: NodePosition::crypto_tree_prefix().clone(),
        };
        let json: OwnCommitmentJson = OwnCommitmentJson::from(own_commitment.clone());
        let reverse = serde_json::to_string(&json).unwrap();
        let own_com_json: OwnCommitmentJson = serde_json::from_str(&reverse).unwrap();
        let own_com: OwnCommitment = OwnCommitment::from(own_com_json);
        assert_eq!(own_com.secret_randomness.clone(), own_commitment.secret_randomness.clone());
        assert_eq!(own_com.image.clone(), own_commitment.image.clone());
        assert_eq!(own_com.position.clone(), own_commitment.position.clone());
        assert_eq!(own_com.commitment.clone(), own_commitment.commitment.clone());
    }

    #[test]
    fn commitmetn_hint_node_format() {
        let secret1 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let (r, a) = interactive_prover::first_message();
        let own_commitment = OwnCommitment
        {
            image: SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
            secret_randomness: r,
            commitment: FirstProverMessage::FirstDlogProverMessage(
                a.clone()
            ),
            position: NodePosition::crypto_tree_prefix().clone(),
        };

        let json: CommitmentHintJson = CommitmentHintJson::from(CommitmentHint::OwnCommitment(own_commitment.clone()));
        println!("{}", serde_json::to_string(&json).unwrap());
        let reverse = serde_json::to_string(&json).unwrap();
        let own_com_json: CommitmentHintJson = serde_json::from_str(&reverse).unwrap();
        let own_com = CommitmentHint::try_from(own_com_json).unwrap();
        match own_com {
            CommitmentHint::OwnCommitment(commitment) => {
                assert_eq!(commitment.secret_randomness.clone(), own_commitment.secret_randomness.clone());
            }
            CommitmentHint::RealCommitment(_) => { panic!("test should not reach here") }
            CommitmentHint::SimulatedCommitment(_) => { panic!("test should not reach here") }
        }
    }
}

