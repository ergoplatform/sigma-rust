//! Ergo peer-to-peer networking

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::wildcard_enum_match_arm)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

mod client;
mod error;
mod message;
mod peer_addr;
mod peer_connection_handler;
mod peer_connector;
mod peer_database;
mod peer_feature;
mod peer_info;
mod peer_spec;
mod protocol_version;

pub use client::Client;
pub use peer_addr::PeerAddr;
pub use peer_database::{in_memory::InMemoryPeerDatabase, PeerDatabase, PeerDatabaseError};
pub use peer_feature::{LocalAddressPeerFeature, PeerFeature, PeerFeatureId};
pub use peer_info::PeerInfo;
pub use peer_spec::PeerSpec;
pub use protocol_version::ProtocolVersion;
