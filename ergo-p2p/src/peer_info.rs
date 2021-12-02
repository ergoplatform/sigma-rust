//! Peer info types

use crate::{peer_addr::PeerAddr, peer_spec::PeerSpec, protocol_version::ProtocolVersion};

/// Direction of the connection to a peer
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ConnectionDirection {
    /// A peer is connecting to us
    Incoming,
    /// We are connecting to a peer
    Outgoing,
}

/// Information about peer to be stored in PeerDatabase
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct PeerInfo {
    /// general information about the peer
    peer_spec: PeerSpec,
    /// timestamp when last handshake was done
    last_handshake: u64,
    /// type of connection (Incoming/Outgoing) established to this peer if any
    conn_type: Option<ConnectionDirection>,
}

impl PeerInfo {
    /// Create new PeerInfo instance
    pub fn new(
        peer_spec: PeerSpec,
        last_handshake: u64,
        conn_type: Option<ConnectionDirection>,
    ) -> Self {
        Self {
            peer_spec,
            last_handshake,
            conn_type,
        }
    }

    /// Return the PeerSpec associated with this PeerInfo
    pub fn spec(&self) -> PeerSpec {
        self.peer_spec.clone()
    }

    /// Create peer info from address only, when we don't know other fields
    /// (e.g. we got this information from config or from API)
    pub fn from_addr(addr: PeerAddr) -> PeerInfo {
        let peer_spec = PeerSpec::new(
            "unknown",
            ProtocolVersion::INITIAL,
            &format!("unknown-{}", addr.to_string()),
            Some(addr),
            None,
        );

        PeerInfo::new(peer_spec, 0, None)
    }
}

/// Arbitrary
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    use proptest::prelude::{Arbitrary, BoxedStrategy};
    use proptest::{option, prelude::*};

    impl Arbitrary for PeerInfo {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (
                any::<PeerSpec>(),
                any::<u64>(),
                option::of(prop_oneof![
                    Just(ConnectionDirection::Incoming),
                    Just(ConnectionDirection::Outgoing)
                ]),
            )
                .prop_map(|(spec, timestamp, direction)| PeerInfo::new(spec, timestamp, direction))
                .boxed()
        }
    }

    impl PeerInfo {
        /// Ensure the PeerSpec has a valid addr
        /// This can happen if declared_addr is none and there is no LocalAddressPeerFeature in features
        pub fn with_ensured_addr(mut self) -> Self {
            if self.peer_spec.addr().is_none() {
                self.peer_spec = self.peer_spec.with_ensured_addr();
            }
            self
        }
    }
}
