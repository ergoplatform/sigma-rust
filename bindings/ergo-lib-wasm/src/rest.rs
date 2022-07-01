//! Wasm API for ergo-rest crate

#[cfg(target_arch = "wasm32")]
pub mod api;
#[cfg(target_arch = "wasm32")]
pub mod chrome_peer_discovery_scan;
pub mod node_conf;
pub mod node_info;
