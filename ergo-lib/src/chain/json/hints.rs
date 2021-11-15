use k256::{FieldBytes, Scalar};
use serde::{Deserialize, Serialize};
use crate::ergotree_interpreter::sigma_protocol::prover::hint::OwnCommitment;
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
// impl Serialize for Scalar{
//     fn serialize_field<T: ?Sized>(
//       &mut self,
//       key: &'static str,
//       value: &T,
//     ) -> Result<(), Self::Error>
//         where
//             T: Serialize;
//
// }

impl From<OwnCommitment> for OwnCommitmentJson {
    fn from(v: OwnCommitment) -> Self {
        let ec_point=&hex::encode(v.image.clone().sigma_serialize_bytes().unwrap().as_slice())[2..].to_string();

        OwnCommitmentJson {
            // secret:Base16EncodedBytes::new(v.secret_randomness.to_bytes().as_slice()).0,
            secret:hex::encode(v.secret_randomness.clone().to_bytes().as_slice()),
            // secret:v.secret_randomness.clone(),
            // secret:BigUint::from_bytes_be(v.secret_randomness.to_bytes().as_slice()).to_str_radix(10),
            position:v.position.positions.clone().into_iter().map(|d| std::char::from_digit(d as u32,10).unwrap()).collect(),
            a:hex::encode(v.commitment.clone().bytes().as_slice()),
            image:ec_point.clone(),

            // image:hex::encode(v.image.clone().sigma_serialize_bytes().unwrap().as_slice()),
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
            // image:SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog::new(v.image))
        }
    }
}

impl From<OwnCommitmentJson> for OwnCommitment{
    fn from(v:OwnCommitmentJson)->Self{
        // println!("{}",v.image.clone());
        // println!("{}",EcPoint::from_base16_str("02a65551523e09530bc8de55d426ff60aace7388cef08f5477c8726713f68c0e78".to_string()).unwrap().sigma_serialize_bytes().unwrap().as_slice().to_vec()[0]);
        OwnCommitment{
            image:SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(v.image.clone()).unwrap()))),
            // secret_randomness:Scalar(ScalarImpl::from_bytes_reduced(hex::decode(v.secret.clone()).unwrap().as_slice())),
            // secret_randomness:Scalar::from(GroupSizedBytes(hex::decode(v.secret.clone()).unwrap().as_slice().into())),
            secret_randomness:Scalar::from_bytes_reduced(hex::decode(v.secret.clone()).unwrap().as_slice().into()),
            position:NodePosition{positions:v.position.clone().chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect()},
            commitment:FirstProverMessage::FirstDlogProverMessage(FirstDlogProverMessage::from(EcPoint::from_base16_str(v.a.clone()).unwrap())),
        }

    }
}
// impl From<>
