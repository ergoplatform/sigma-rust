use k256::{FieldBytes, Scalar};
use serde::{Deserialize, Serialize};
use crate::ergotree_interpreter::sigma_protocol::prover::hint::{OwnCommitment, SimulatedCommitment};
use num_bigint::BigUint;
use ergotree_interpreter::sigma_protocol::prover::hint::RealCommitment;
use ergotree_ir::chain::base16_bytes::{Base16DecodedBytes, Base16EncodedBytes};
use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean::ProofOfKnowledge;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree::{ProveDhTuple, ProveDlog};
use crate::ergotree_interpreter::sigma_protocol::{FirstProverMessage, ProverMessage};
use crate::ergotree_ir::serialization::SigmaSerializable;
use crate::ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use ergotree_interpreter::sigma_protocol::dlog_protocol::FirstDlogProverMessage;
use ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;

use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog as OtherProveDlog;

#[derive(Serialize,Deserialize)]
pub struct OwnCommitmentJson {
    pub secret:String,
    pub position:String,
    pub a:String,
    pub image:String,
}

pub struct RealCommitmentJson{
    pub image:String,
    pub position:String,
    pub a:String,
}

pub struct SimulatedCommitmentJson{
    pub image:String,
    pub position:String,
    pub a:String,
}

impl From<OwnCommitment> for OwnCommitmentJson {
    fn from(v: OwnCommitment) -> Self {
        let ec_point=&hex::encode(v.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string();

        OwnCommitmentJson {
            secret:hex::encode(v.secret_randomness.clone().to_bytes().as_slice()),
            position:v.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32,10).unwrap()).collect(),
            a:hex::encode(v.commitment.clone().bytes().as_slice()),
            image:ec_point.clone(),
        }
    }
}

impl From<RealCommitment> for RealCommitmentJson{
    fn from(v: RealCommitment) -> Self {
        let ec_point=&hex::encode(v.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string();
        RealCommitmentJson {
            position:v.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32,10).unwrap()).collect(),
            a:hex::encode(v.commitment.clone().bytes().as_slice()),
            image:ec_point.clone(),
        }
    }
}

impl From<SimulatedCommitment> for SimulatedCommitmentJson{
    fn from(v: SimulatedCommitment) -> Self {
        let ec_point=&hex::encode(v.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string();
        SimulatedCommitmentJson {
            position:v.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32,10).unwrap()).collect(),
            a:hex::encode(v.commitment.clone().bytes().as_slice()),
            image:ec_point.clone(),
        }
    }
}


impl From<OwnCommitmentJson> for OwnCommitment{
    fn from(v:OwnCommitmentJson)->Self{
        OwnCommitment{
            image:SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(v.image.clone()).unwrap()))),
            secret_randomness:Scalar::from_bytes_reduced(hex::decode(v.secret.clone()).unwrap().as_slice().into()),
            position:NodePosition{positions:v.position.clone().chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect()},
            commitment:FirstProverMessage::FirstDlogProverMessage(FirstDlogProverMessage::from(EcPoint::from_base16_str(v.a.clone()).unwrap())),
        }

    }
}

impl From<RealCommitmentJson> for RealCommitment{
    fn from(v:RealCommitmentJson)->Self{
        RealCommitment{
            image:SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(v.image.clone()).unwrap()))),
            position:NodePosition{positions:v.position.clone().chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect()},
            commitment:FirstProverMessage::FirstDlogProverMessage(FirstDlogProverMessage::from(EcPoint::from_base16_str(v.a.clone()).unwrap())),
        }

    }
}

impl From<SimulatedCommitmentJson> for SimulatedCommitment{
    fn from(v:SimulatedCommitmentJson)->Self{
        SimulatedCommitment{
            image:SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(v.image.clone()).unwrap()))),
            position:NodePosition{positions:v.position.clone().chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect()},
            commitment:FirstProverMessage::FirstDlogProverMessage(FirstDlogProverMessage::from(EcPoint::from_base16_str(v.a.clone()).unwrap())),
        }

    }
}
#[cfg(test)]
mod tests{
    use crate::chain::json::hints::OwnCommitmentJson;
    use crate::ergotree_interpreter::sigma_protocol::dlog_protocol::interactive_prover;
    use crate::ergotree_interpreter::sigma_protocol::FirstProverMessage;
    use crate::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
    use crate::ergotree_interpreter::sigma_protocol::unproven_tree::NodePosition;
    use crate::ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProofOfKnowledgeTree};
    use crate::ergotree_interpreter::sigma_protocol::prover::hint::{OwnCommitment, SimulatedCommitment};

    #[test]
    fn round_trip_own_commitment(){
        let secret1 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let (r, a) = interactive_prover::first_message();
        let own_commitment= OwnCommitment
        {
            image: SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk1.clone())),
            secret_randomness: r,
            commitment: FirstProverMessage::FirstDlogProverMessage(
                a.clone()
            ),
            position: NodePosition::crypto_tree_prefix().clone(),
        };
        let json:OwnCommitmentJson=OwnCommitmentJson::from(own_commitment.clone());
        let reverse=serde_json::to_string(&json).unwrap();
        let own_com_json:OwnCommitmentJson=serde_json::from_str(&reverse).unwrap();
        let own_com:OwnCommitment=OwnCommitment::from(own_com_json);
        assert_eq!(own_com.secret_randomness.clone(),own_commitment.secret_randomness.clone());
        assert_eq!(own_com.image.clone(),own_commitment.image.clone());
        assert_eq!(own_com.position.clone(),own_commitment.position.clone());
        assert_eq!(own_com.commitment.clone(),own_commitment.commitment.clone());
    }
}

