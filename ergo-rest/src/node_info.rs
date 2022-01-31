use serde::{Deserialize, Serialize};

/// Node extended information from /info REST API endpoint
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NodeInfo {
    /// Node name
    pub name: String,
}
