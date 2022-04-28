//! Generate Ergo blockchains for simulation and testing

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
//#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use ergo_lib::ergotree_ir::chain::header::Header;
use ergo_lib::{
    ergo_chain_types::ExtensionCandidate,
    wallet::{ext_secret_key::ExtSecretKey, mnemonic::Mnemonic},
};
use ergo_nipopow::NipopowAlgos;
use num_bigint::{BigInt, Sign};

pub mod chain_generation;
mod fake_pow_scheme;

/// Ergo block
#[derive(Clone, Debug)]
pub struct ErgoFullBlock {
    pub(crate) header: Header,
    //block_transactions: BlockTransactions,
    pub(crate) extension: ExtensionCandidate,
    //ad_proofs: ProofBytes,
}

impl std::convert::TryInto<ergo_nipopow::PoPowHeader> for ErgoFullBlock {
    type Error = &'static str;
    fn try_into(self) -> Result<ergo_nipopow::PoPowHeader, &'static str> {
        let interlinks_proof = match NipopowAlgos::proof_for_interlink_vector(&self.extension) {
            Some(proof) => proof,
            None => return Err("Unable to generate BatchMerkleProof for interlinks"),
        };
        let interlinks = NipopowAlgos::unpack_interlinks(&self.extension)?;
        Ok(ergo_nipopow::PoPowHeader {
            header: self.header,
            interlinks,
            interlinks_proof,
        })
    }
}

/// Returns the secret key of the miner secret with its `BigInt` representation. Taken from ergo
/// test suite.
pub(crate) fn default_miner_secret() -> (ExtSecretKey, BigInt) {
    let test_mnemonic =
        "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic";
    let seed = Mnemonic::to_seed(test_mnemonic, "");
    let default_root_secret = ExtSecretKey::derive_master(seed).unwrap();
    let bytes = default_root_secret.secret_key_bytes();
    (
        default_root_secret,
        BigInt::from_bytes_be(Sign::Plus, &bytes),
    )
}
