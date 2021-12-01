//! Peer database types

pub mod in_memory;

use std::collections::HashMap;

use crate::{peer_addr::PeerAddr, peer_info::PeerInfo};

type PeerMap = HashMap<PeerAddr, PeerInfo>;

/// PeerDatabase errors
#[derive(PartialEq, Eq, Debug)]
pub enum PeerDatabaseError {
    /// Peer was not found in the database
    NotFound,
    /// The peer had no address associated with it
    /// The node spec had no declared address and the node isn't using the LocalAddress peer feature
    NoPeerAddr,
}

/// Peer database trait
/// Common operations for managing peers
pub trait PeerDatabase {
    /// Get `PeerInfo` based on the address
    fn get_peer_by_addr(&self, addr: PeerAddr) -> Option<PeerInfo>;

    /// Add a peer to the database
    /// If a peer with the address already exists then it is updated and the old value is returned
    /// If the database did not contain the peer info then None is returned
    /// If the peer has no associated address PeerDatabaseError::NoPeerAddr is returned
    fn add_or_update_peer(
        &mut self,
        peer_info: PeerInfo,
    ) -> Result<Option<PeerInfo>, PeerDatabaseError>;

    /// Remove peer from the database
    /// Returns the peer if it existed in the database
    fn remove_peer(&mut self, addr: PeerAddr) -> Result<(), PeerDatabaseError>;

    /// Return a mapping of peer address -> peer info
    fn known_peers(&self) -> PeerMap;
}
