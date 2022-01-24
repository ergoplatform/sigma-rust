use serde::{Deserialize, Serialize};

use crate::NodeResponse;

/// Node extended information from /info REST API endpoint
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[repr(C)]
pub struct NodeInfo {
    /// Node name
    pub name: String,
}

impl NodeResponse for NodeInfo {}
