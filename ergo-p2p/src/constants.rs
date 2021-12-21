use std::time::Duration;

/// The timeout for handshakes when connecting to new peers.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(4);
