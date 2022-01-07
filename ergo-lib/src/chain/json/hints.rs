//! Hints JSON wrapper

use crate::ergotree_interpreter::sigma_protocol::prover::hint::{
    CommitmentHint, OwnCommitment, SimulatedCommitment,
};
use crate::ergotree_interpreter::sigma_protocol::{FirstProverMessage, ProverMessage};
use crate::ergotree_ir::serialization::SigmaSerializable;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use ergotree_interpreter::sigma_protocol::dlog_protocol::FirstDlogProverMessage;
use ergotree_interpreter::sigma_protocol::prover::hint::RealCommitment;
use ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog as OtherProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use k256::elliptic_curve::generic_array::GenericArray;
use k256::elliptic_curve::ops::Reduce;
use k256::{Scalar, U256};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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
    pub fn image(&self) -> Result<SigmaBoolean, &'static str> {
        match EcPoint::from_base16_str(self.pubkey.h.clone()) {
            Some(point) => Ok(SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(point)),
            )),
            None => Err("invalid ECPoint"),
        }
    }

    /// Extracts position(NodePosition)
    pub fn position(&self) -> Result<NodePosition, &'static str> {
        let positions: Vec<&str> = self.position.split('-').collect();
        let mut position: Vec<usize> = vec![];
        for pos in positions {
            match pos.parse::<usize>() {
                Ok(value) => {
                    position.push(value);
                }
                Err(_) => {
                    return Err("not a node position format");
                }
            }
        }
        Ok(NodePosition {
            positions: position,
        })
    }

    /// Extracts commitment(FirstProverMessage)
    pub fn commitment(&self) -> Result<FirstProverMessage, &'static str> {
        match EcPoint::from_base16_str(self.a.clone()) {
            None => Err("invalid ECPoint"),
            Some(point) => Ok(FirstProverMessage::FirstDlogProverMessage(
                FirstDlogProverMessage::from(point),
            )),
        }
    }
}

// In all unwrap use cases the option have Some value.
#[allow(clippy::unwrap_used)]
impl From<CommitmentHint> for CommitmentHintJson {
    fn from(v: CommitmentHint) -> Self {
        let mut _hint: Option<String> = None;
        let mut secret: Option<String> = None;
        let mut _a: Option<String> = None;
        let proof_type = "dlog".to_string();
        let mut _position: Option<String> = None;
        let mut _ec_point: Option<String> = None;
        match v {
            CommitmentHint::OwnCommitment(cmt) => {
                _hint = Some("cmtWithSecret".to_string());
                secret = Some(hex::encode(
                    cmt.secret_randomness.clone().to_bytes().as_slice(),
                ));
                _a = Some(hex::encode(cmt.commitment.bytes().as_slice()));
                _position = Some(
                    cmt.position
                        .positions
                        .clone()
                        .into_iter()
                        .map(|d| std::char::from_digit(d as u32, 10).unwrap().to_string())
                        .collect::<Vec<_>>()
                        .join("-"),
                );
                _ec_point = Some(
                    hex::encode(cmt.image.sigma_serialize_bytes().unwrap().as_slice())[2..]
                        .to_string(),
                );
            }
            CommitmentHint::RealCommitment(cmt) => {
                _hint = Some("cmtReal".to_string());
                _a = Some(hex::encode(cmt.commitment.bytes().as_slice()));
                _position = Some(
                    cmt.position
                        .positions
                        .clone()
                        .into_iter()
                        .map(|d| std::char::from_digit(d as u32, 10).unwrap().to_string())
                        .collect::<Vec<_>>()
                        .join("-"),
                );
                _ec_point = Some(
                    hex::encode(cmt.image.sigma_serialize_bytes().unwrap().as_slice())[2..]
                        .to_string(),
                );
            }
            CommitmentHint::SimulatedCommitment(cmt) => {
                _hint = Some("cmtSimulated".to_string());
                _a = Some(hex::encode(cmt.commitment.bytes().as_slice()));
                _position = Some(
                    cmt.position
                        .positions
                        .clone()
                        .into_iter()
                        .map(|d| std::char::from_digit(d as u32, 10).unwrap().to_string())
                        .collect::<Vec<_>>()
                        .join("-"),
                );
                _ec_point = Some(
                    hex::encode(cmt.image.sigma_serialize_bytes().unwrap().as_slice())[2..]
                        .to_string(),
                );
            }
        }

        let public_key = PublicKeyJson {
            op: -51,
            h: _ec_point.unwrap(),
        };

        CommitmentHintJson {
            hint: _hint.unwrap(),
            pubkey: public_key,
            position: _position.unwrap(),
            proof_type,
            a: _a.unwrap(),
            secret,
        }
    }
}

// All unwarp use cases handled
#[allow(clippy::unwrap_used)]
impl TryFrom<CommitmentHintJson> for CommitmentHint {
    type Error = &'static str;

    fn try_from(v: CommitmentHintJson) -> Result<CommitmentHint, &'static str> {
        let image: Option<SigmaBoolean>;
        let position: Option<NodePosition>;
        let commitment: Option<FirstProverMessage>;
        match v.image() {
            Ok(img) => image = Some(img),
            Err(e) => {
                return Err(e);
            }
        }
        match v.position() {
            Ok(pos) => position = Some(pos),
            Err(e) => {
                return Err(e);
            }
        }
        match v.commitment() {
            Ok(cmt) => commitment = Some(cmt),
            Err(e) => {
                return Err(e);
            }
        }

        match v.hint.as_ref() {
            "cmtWithSecret" => match v.secret.clone() {
                None => Err("There is no secret"),
                Some(secret) => match hex::decode(secret) {
                    Ok(decoded_secret) => Ok(CommitmentHint::OwnCommitment(OwnCommitment {
                        secret_randomness: <Scalar as Reduce<U256>>::from_be_bytes_reduced(
                            GenericArray::clone_from_slice(decoded_secret.as_slice()),
                        ),
                        image: image.unwrap(),
                        position: position.unwrap(),
                        commitment: commitment.unwrap(),
                    })),
                    Err(_) => Err("Not a valid secret"),
                },
            },
            "cmtReal" => Ok(CommitmentHint::RealCommitment(RealCommitment {
                image: image.unwrap(),
                position: position.unwrap(),
                commitment: commitment.unwrap(),
            })),
            "cmtSimulated" => Ok(CommitmentHint::SimulatedCommitment(SimulatedCommitment {
                image: image.unwrap(),
                position: position.unwrap(),
                commitment: commitment.unwrap(),
            })),
            _ => Err("invalid header length"),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use crate::chain::json::hints::CommitmentHintJson;
    use crate::ergotree_interpreter::sigma_protocol::dlog_protocol::interactive_prover;
    use crate::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
    use crate::ergotree_interpreter::sigma_protocol::prover::hint::OwnCommitment;
    use crate::ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
    use crate::ergotree_interpreter::sigma_protocol::FirstProverMessage;
    use crate::ergotree_ir::sigma_protocol::sigma_boolean::{
        SigmaBoolean, SigmaProofOfKnowledgeTree,
    };
    use ergotree_interpreter::sigma_protocol::prover::hint::CommitmentHint;
    use std::convert::TryFrom;

    #[test]
    fn commitment_hint_node_format() {
        let secret1 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let (r, a) = interactive_prover::first_message();
        let own_commitment = OwnCommitment {
            image: SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1)),
            secret_randomness: r,
            commitment: FirstProverMessage::FirstDlogProverMessage(a),
            position: NodePosition::crypto_tree_prefix(),
        };

        let json: CommitmentHintJson =
            CommitmentHintJson::from(CommitmentHint::OwnCommitment(own_commitment.clone()));
        let reverse = serde_json::to_string(&json).unwrap();
        let own_com_json: CommitmentHintJson = serde_json::from_str(&reverse).unwrap();
        let own_com = CommitmentHint::try_from(own_com_json).unwrap();
        match own_com {
            CommitmentHint::OwnCommitment(commitment) => {
                assert_eq!(
                    commitment.secret_randomness.clone(),
                    own_commitment.secret_randomness.clone()
                );
            }
            CommitmentHint::RealCommitment(_) => {
                panic!("test should not reach here")
            }
            CommitmentHint::SimulatedCommitment(_) => {
                panic!("test should not reach here")
            }
        }
    }
}
