use std::convert::TryFrom;
use std::num::ParseIntError;
use std::str::FromStr;

use ergo_chain_types::Base16DecodedBytes;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use serde::Deserialize;
use serde::Serialize;

use crate::sigma_protocol::challenge::Challenge;
use crate::sigma_protocol::prover::hint::RealSecretProof;
use crate::sigma_protocol::prover::hint::SimulatedSecretProof;
use crate::sigma_protocol::sig_serializer::parse_sig_compute_challenges;
use crate::sigma_protocol::sig_serializer::SigParsingError;
use crate::sigma_protocol::unproven_tree::NodePosition;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CommitmentHintJson {}

#[derive(Deserialize, Serialize)]
pub struct NodePositionJson(String);

impl From<NodePosition> for NodePositionJson {
    fn from(p: NodePosition) -> Self {
        NodePositionJson(
            p.positions
                .into_iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join("-"),
        )
    }
}

impl TryFrom<NodePositionJson> for NodePosition {
    type Error = ParseIntError;

    fn try_from(pj: NodePositionJson) -> Result<Self, Self::Error> {
        Ok(NodePosition {
            positions: pj
                .0
                .split('-')
                .map(usize::from_str)
                .collect::<Result<Vec<usize>, _>>()?,
        })
    }
}

#[derive(Deserialize)]
pub struct RealSecretProofJson {
    #[cfg_attr(feature = "json", serde(rename = "pubkey"))]
    pub image: SigmaBoolean,
    #[cfg_attr(feature = "json", serde(rename = "challenge"))]
    pub challenge: Challenge,
    #[cfg_attr(feature = "json", serde(rename = "proof"))]
    pub unchecked_tree_bytes: Base16DecodedBytes,
    #[cfg_attr(feature = "json", serde(rename = "position"))]
    pub position: NodePosition,
}

impl TryFrom<RealSecretProofJson> for RealSecretProof {
    type Error = SigParsingError;

    fn try_from(rj: RealSecretProofJson) -> Result<Self, Self::Error> {
        let unchecked_tree =
            parse_sig_compute_challenges(&rj.image, rj.unchecked_tree_bytes.into())?;
        Ok(RealSecretProof {
            image: rj.image,
            challenge: rj.challenge,
            unchecked_tree,
            position: rj.position,
        })
    }
}

#[derive(Deserialize)]
pub struct SimulatedSecretProofJson {
    #[cfg_attr(feature = "json", serde(rename = "pubkey"))]
    pub image: SigmaBoolean,
    #[cfg_attr(feature = "json", serde(rename = "challenge"))]
    pub challenge: Challenge,
    #[cfg_attr(feature = "json", serde(rename = "proof"))]
    pub unchecked_tree_bytes: Base16DecodedBytes,
    #[cfg_attr(feature = "json", serde(rename = "position"))]
    pub position: NodePosition,
}

impl TryFrom<SimulatedSecretProofJson> for SimulatedSecretProof {
    type Error = SigParsingError;

    fn try_from(sj: SimulatedSecretProofJson) -> Result<Self, Self::Error> {
        let unchecked_tree =
            parse_sig_compute_challenges(&sj.image, sj.unchecked_tree_bytes.into())?;
        Ok(SimulatedSecretProof {
            image: sj.image,
            challenge: sj.challenge,
            unchecked_tree,
            position: sj.position,
        })
    }
}
