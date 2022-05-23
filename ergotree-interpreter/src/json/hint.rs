use std::convert::TryFrom;

use serde::Deserialize;
use serde::Serialize;

use crate::sigma_protocol::unproven_tree::NodePosition;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CommitmentHintJson {}

#[derive(Deserialize, Serialize)]
pub struct NodePositionJson(String);

impl From<NodePosition> for NodePositionJson {
    fn from(_: NodePosition) -> Self {
        todo!()
    }
}

impl TryFrom<NodePositionJson> for NodePosition {
    type Error = String;

    fn try_from(value: NodePositionJson) -> Result<Self, Self::Error> {
        todo!()
    }
}
