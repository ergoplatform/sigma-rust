//! Peer database types

use std::collections::HashMap;

use crate::{peer_addr::PeerAddr, peer_info::PeerInfo};

type PeerMap = HashMap<PeerAddr, PeerInfo>;

trait PeerDatabase {
    fn get_peer_by_addr(&self, addr: PeerAddr) -> Option<&PeerInfo>;

    fn add_or_update_peer(&mut self, peer_info: PeerInfo);

    fn remove_peer(&mut self, addr: PeerAddr);

    fn known_peers(&self) -> PeerMap;
}

/// In-memory peer database implementation
#[derive(Default)]
pub struct InMemoryPeerDatabase {
    peers: PeerMap,
}

impl PeerDatabase for InMemoryPeerDatabase {
    fn get_peer_by_addr(&self, addr: PeerAddr) -> Option<&PeerInfo> {
        self.peers.get(&addr)
    }

    fn add_or_update_peer(&mut self, peer_info: PeerInfo) {
        if let Some(addr) = peer_info.peer_spec.addr() {
            self.peers.insert(addr, peer_info);
        }
    }

    fn remove_peer(&mut self, addr: PeerAddr) {
        self.peers.remove(&addr);
    }

    fn known_peers(&self) -> PeerMap {
        self.peers.clone()
    }
}
