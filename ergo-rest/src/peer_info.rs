use ergo_chain_types::ConnectionDirection;
use ergo_chain_types::PeerAddr;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
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
    match s.parse().map_err(D::Error::custom) {
        Ok(peer_addr) => Ok(peer_addr),
        Err(_) => {
            // try to fix an invalid IP v6 address encoding on node
            // the format should be `[ipv6]:port`, but pre-JDK14 node returns `ipv6:port`
            // see https://bugs.java.com/bugdatabase/view_bug.do?bug_id=JDK-8225499
            //dbg!(&s);
            let parts: Vec<&str> = s.rsplit(':').collect();
            if parts.len() == 2 {
                // not an IP v6 address
                return Err(D::Error::custom(format!(
                    "expected invalid IP v6(without brackets) address, got: {}",
                    s
                )));
            }
            #[allow(clippy::unwrap_used)]
            let port = parts.first().cloned().unwrap();
            let host: String = parts
                .into_iter()
                .skip(1)
                .rev()
                .collect::<Vec<&str>>()
                .join(":");
            // put host inside brackets to make it a valid IP v6 address
            let str = format!("[{}]:{}", host, port);
            str.parse().map_err(D::Error::custom)
        }
    }
}

#[cfg(test)]
#[cfg(feature = "json")]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_peer_info() {
        let json = "{\n    \"address\" : \"/62.106.112.158:9030\",\n    \"lastMessage\" : 0,\n    \"lastHandshake\" : 0,\n    \"name\" : \"ergo-mainnet-4.0.12\",\n    \"connectionType\" : null\n  }";
        let _: PeerInfo = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn test_parse_peer_info_ipv6_pre_jkd14() {
        let json = "{\n    \"address\" : \"/2a0d:6fc0:7cb:be00:50be:7d74:7a00:aa3e:9030\",\n    \"lastMessage\" : 0,\n    \"lastHandshake\" : 0,\n    \"name\" : \"ergo-mainnet-4.0.12\",\n    \"connectionType\" : null\n  }";
        let _: PeerInfo = serde_json::from_str(json).unwrap();
    }
}
