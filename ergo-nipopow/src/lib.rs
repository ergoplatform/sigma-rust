//! Ergo NiPoPoW implementation
//! See FC 20 (published) version <https://fc20.ifca.ai/preproceedings/74.pdf>

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

mod autolykos_pow_scheme;
mod nipopow_algos;
mod nipopow_proof;
mod nipopow_verifier;

pub use nipopow_algos::{decode_compact_bits, NipopowAlgos, INTERLINK_VECTOR_PREFIX};
pub use nipopow_proof::{NipopowProof, NipopowProofError, PoPowHeader};
pub use nipopow_verifier::NipopowVerifier;
