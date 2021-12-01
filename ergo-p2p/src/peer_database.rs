//! Peer database types

use std::collections::HashMap;

use crate::{peer_addr::PeerAddr, peer_info::PeerInfo};

type PeerMap = HashMap<PeerAddr, PeerInfo>;

/// PeerDatabase errors
pub enum PeerDatabaseError {
    /// Peer was not found in the database
    NotFound,
}

/// Peer database trait
/// Common operations for managing peers
pub trait PeerDatabase {
    /// Get `PeerInfo` based on the address
    fn get_peer_by_addr(&self, addr: PeerAddr) -> Option<&PeerInfo>;

    /// Add a peer to the database
    /// If a peer with the address already exists then it is updated and the old value is returned
    /// If the database did not contain the peer info then None is returned
    fn add_or_update_peer(&mut self, peer_info: PeerInfo) -> Option<PeerInfo>;

    /// Remove peer from the database
    /// Returns the peer if it existed in the database
    fn remove_peer(&mut self, addr: PeerAddr) -> Option<PeerInfo>;

    /// Return a mapping of peer address -> peer info
    fn known_peers(&self) -> PeerMap;
}

/// In-memory peer database implementation
#[derive(Default, Debug)]
pub struct InMemoryPeerDatabase {
    peers: PeerMap,
}

impl InMemoryPeerDatabase {
    /// Create a new InMemory peer database from an existing hashmap
    pub fn new(peers: PeerMap) -> InMemoryPeerDatabase {
        Self { peers }
    }
}

impl PeerDatabase for InMemoryPeerDatabase {
    fn get_peer_by_addr(&self, addr: PeerAddr) -> Option<&PeerInfo> {
        self.peers.get(&addr)
    }

    fn add_or_update_peer(&mut self, peer_info: PeerInfo) -> Option<PeerInfo> {
        if let Some(addr) = peer_info.peer_spec.addr() {
            self.peers.insert(addr, peer_info)
        } else {
            None
        }
    }

    fn remove_peer(&mut self, addr: PeerAddr) -> Option<PeerInfo> {
        self.peers.remove(&addr)
    }

    fn known_peers(&self) -> PeerMap {
        self.peers.clone()
    }
}

/// Arbitrary
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    use proptest::collection::hash_map;
    use proptest::prelude::*;
    use proptest::prelude::{Arbitrary, BoxedStrategy};

    impl Arbitrary for InMemoryPeerDatabase {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (hash_map(any::<PeerAddr>(), any::<PeerInfo>(), 1..5))
                .prop_map(Self::new)
                .boxed()
        }
    }

    impl InMemoryPeerDatabase {
        /// Seed database with a peer
        pub fn with_peer(mut self, info: PeerInfo) -> InMemoryPeerDatabase {
            self.add_or_update_peer(info);
            self
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    // use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    #[test]
    fn get_peer_by_addr() {
        let info = force_any_val::<PeerInfo>();
        let db = force_any_val::<InMemoryPeerDatabase>().with_peer(info.clone());
        let result = db.get_peer_by_addr(info.peer_spec.addr().unwrap()).unwrap();

        assert_eq!(&info, result);
    }

    // Test adding peer in the case its not already in the db
    #[test]
    fn add_or_update_peer_doesnt_exist() {
        let mut db = force_any_val::<InMemoryPeerDatabase>();
        let info = force_any_val::<PeerInfo>();
        let result = db.add_or_update_peer(info);

        assert!(result.is_some())
    }

    // Test adding peer in the case it already exists
    #[test]
    fn add_or_update_peer_exists() {
        let info = force_any_val::<PeerInfo>();
        let mut db = force_any_val::<InMemoryPeerDatabase>().with_peer(info.clone());
        // clone info and change the node_name
        let result = db.add_or_update_peer(info);

        assert!(result.is_some())
    }
}
