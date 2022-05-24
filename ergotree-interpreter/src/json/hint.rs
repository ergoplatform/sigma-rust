use std::convert::TryFrom;
use std::num::ParseIntError;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

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
