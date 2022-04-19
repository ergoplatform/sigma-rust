use ergo_chain_types::ConnectionDirection;
use ergo_chain_types::PeerAddr;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[repr(C)]
/// Peer info returned by node REST API
pub struct PeerInfo {
    /// Peer address
    #[serde(
        rename = "address",
        deserialize_with = "parse_peer_addr_with_leading_slash"
    )]
    pub addr: PeerAddr,
    /// Last message
    #[serde(rename = "lastMessage")]
    pub last_message: u64,
    /// Timestamp of the last handshake
    #[serde(rename = "lastHandshake")]
    pub last_handshake: u64, // TODO: any more precise type?
    /// Node name
    pub name: String,
    /// Type of connection (Incoming/Outgoing) established to this peer if any
    #[serde(rename = "connectionType")]
    pub conn_type: Option<ConnectionDirection>,
}

/// The `PeerAddr` reported by ergo nodes at the `peers/all` endpoint begin with a leading '/'.
/// This custom deserialization function simply removes the leading character then uses the
/// serde-generated function to parse.
fn parse_peer_addr_with_leading_slash<'de, D>(deserializer: D) -> Result<PeerAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    let s = if s.starts_with('/') {
        let mut chars = s.chars();
        chars.next();
        chars.as_str()
    } else {
        s
    };
    s.parse().map_err(D::Error::custom)
}

#[cfg(test)]
#[cfg(feature = "json")]
mod tests {
    use super::*;
    #[test]
    fn test_parse_peer_info() {
        let json = "{\n    \"address\" : \"/62.106.112.158:9030\",\n    \"lastMessage\" : 0,\n    \"lastHandshake\" : 0,\n    \"name\" : \"ergo-mainnet-4.0.12\",\n    \"connectionType\" : null\n  }";
        let _: PeerInfo = serde_json::from_str(json).unwrap();
    }
}
