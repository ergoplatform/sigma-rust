use crate::{peer_addr::PeerAddr, peer_info::PeerInfo};

use super::{PeerDatabase, PeerDatabaseError, PeerMap};

/// In-memory peer database implementation
#[derive(Default, Debug)]
pub struct InMemoryPeerDatabase {
    peers: PeerMap,
}

impl PeerDatabase for InMemoryPeerDatabase {
    fn get_peer_by_addr(&self, addr: PeerAddr) -> Option<PeerInfo> {
        Some(self.peers.get(&addr)?.clone())
    }

    fn add_or_update_peer(
        &mut self,
        peer_info: PeerInfo,
    ) -> Result<Option<PeerInfo>, PeerDatabaseError> {
        if let Some(addr) = peer_info.spec().addr() {
            Ok(self.peers.insert(addr, peer_info))
        } else {
            Err(PeerDatabaseError::NoPeerAddr)
        }
    }

    fn remove_peer(&mut self, addr: PeerAddr) -> Result<(), PeerDatabaseError> {
        self.peers
            .remove(&addr)
            .ok_or(PeerDatabaseError::NotFound)?;

        Ok(())
    }

    fn known_peers(&self) -> PeerMap {
        self.peers.clone()
    }
}

/// Arbitrary
#[allow(unused_must_use)]
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::prelude::{Arbitrary, BoxedStrategy};

    impl Arbitrary for InMemoryPeerDatabase {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (vec(any::<PeerInfo>(), 1..5))
                .prop_map(|infos| {
                    let mut db = Self::default();
                    for i in infos {
                        db.add_or_update_peer(i);
                    }
                    db
                })
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

    use sigma_test_util::force_any_val;

    #[test]
    fn get_peer_by_addr() {
        let info = force_any_val::<PeerInfo>().with_ensured_addr();
        let db = force_any_val::<InMemoryPeerDatabase>().with_peer(info.clone());
        let result = db.get_peer_by_addr(info.spec().addr().unwrap()).unwrap();

        assert_eq!(info, result);
    }

    // Test adding peer in the case its not already in the db
    #[test]
    fn add_or_update_peer_doesnt_exist() {
        let mut db = force_any_val::<InMemoryPeerDatabase>();
        let info = force_any_val::<PeerInfo>().with_ensured_addr();
        db.add_or_update_peer(info.clone()).unwrap();

        assert!(db.get_peer_by_addr(info.spec().addr().unwrap()).is_some())
    }

    // Test adding peer in the case it already exists
    #[test]
    fn add_or_update_peer_exists() {
        let info = force_any_val::<PeerInfo>().with_ensured_addr();
        let mut db = force_any_val::<InMemoryPeerDatabase>().with_peer(info.clone());
        // clone info and change the node_name
        let result = db.add_or_update_peer(info.clone()).unwrap().unwrap();

        // add or update returned the original peer_info
        assert_eq!(result, info)
    }

    #[test]
    fn remove_peer_db_contains_peer() {
        let info = force_any_val::<PeerInfo>().with_ensured_addr();
        let mut db = force_any_val::<InMemoryPeerDatabase>().with_peer(info.clone());
        let result = db.remove_peer(info.spec().addr().unwrap());

        assert!(result.is_ok());
        assert!(db.get_peer_by_addr(info.spec().addr().unwrap()).is_none())
    }
}
