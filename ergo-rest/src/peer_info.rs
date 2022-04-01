use ergo_chain_types::ConnectionDirection;
use ergo_chain_types::PeerAddr;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[repr(C)]
/// Peer info returned by node REST API
pub struct PeerInfo {
    /// Peer address
    #[serde(rename = "address", deserialize_with = "from_ergo_node_string")]
    pub addr: PeerAddr,
    /// Timestamp of the last handshake
    #[serde(rename = "lastHandshake")]
    pub last_handshake: u64, // TODO: any more precise type?
    /// Node name
    pub name: String,
    /// Type of connection (Incoming/Outgoing) established to this peer if any
    #[serde(rename = "connectionType")]
    pub conn_type: Option<ConnectionDirection>,
}

fn from_ergo_node_string<'de, D>(deserializer: D) -> Result<PeerAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    use sigma_ser::ScorexSerializable;

    // Ergo node reports address with leading '/'. Need to remove it.
    let mut chars = s.chars();
    chars.next();
    let s = chars.as_str();
    PeerAddr::scorex_parse_bytes(s.as_bytes()).map_err(D::Error::custom)
}
