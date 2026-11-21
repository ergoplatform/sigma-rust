//! Trait for reading [`PoPowHeader`]s and [`Header`]s from an external store.
//!
//! Used by [`crate::NipopowAlgos::prove_with_reader`] to construct a NiPoPoW
//! proof without first materializing the entire chain as `PoPowHeader`s.
//!
//! # Relationship to the JVM `ErgoHistoryReader`
//!
//! This trait is the minimal Rust analogue of the subset of methods that
//! `org.ergoplatform.modifiers.history.popow.NipopowProverWithDbAlgs.prove`
//! requires from `ErgoHistoryReader`. Implementations are expected to be
//! cheaply clonable readers backed by some external storage.
//!
//! # Asymptotic motivation
//!
//! [`crate::NipopowAlgos::prove`] requires the caller to materialize the
//! whole chain as `PoPowHeader`s before it runs — `O(N)` `popow_header`
//! constructions per proof, where `N` is the chain length. The internal
//! algorithm only inspects most of those headers' `n_bits` / `id` fields,
//! and reads `interlinks` / `interlinks_proof` only for blocks that land in
//! the final prefix.
//!
//! [`crate::NipopowAlgos::prove_with_reader`] inverts that: it walks the
//! interlink hierarchy starting from the suffix head and only requests
//! `PoPowHeader`s for blocks the walk actually visits — roughly
//! `m + k + m * log2(N)` `popow_header_by_id` calls per proof. For the P2P
//! defaults `m = 6, k = 10` on a `N ~= 270k` chain that's ~120 fetches
//! versus 270k for the in-memory variant: three orders of magnitude fewer.
//!
//! # Consistency expectation
//!
//! Any header `id` returned by one method (for example, the `id` of a
//! `Header` returned by [`PopowHeaderReader::last_headers`] or appearing in
//! the `interlinks` of a `PoPowHeader`) MUST be resolvable via
//! [`PopowHeaderReader::popow_header_by_id`]. Likewise, the genesis block
//! at height `1` MUST be resolvable via
//! [`PopowHeaderReader::popow_header_at_height`]. Inconsistent readers will
//! cause [`crate::NipopowAlgos::prove_with_reader`] to fail with
//! [`crate::NipopowProofError::MissingPopowHeader`].

use ergo_chain_types::{BlockId, Header};

use crate::nipopow_proof::PoPowHeader;

/// Read-only access to a chain's [`PoPowHeader`]s and [`Header`]s for
/// db-backed NiPoPoW proof construction. See the module docs for semantics
/// and the asymptotic motivation.
pub trait PopowHeaderReader {
    /// Returns the current chain height (number of blocks). Used to enforce
    /// the `headers_height >= k + m` precondition before proving.
    fn headers_height(&self) -> u32;

    /// Looks up a [`PoPowHeader`] by its block id. Hot path during the
    /// interlink walk in [`crate::NipopowAlgos::prove_with_reader`].
    /// Returns `None` if the reader has no record of `id`.
    fn popow_header_by_id(&self, id: &BlockId) -> Option<PoPowHeader>;

    /// Looks up a [`PoPowHeader`] by absolute block height (1 = genesis).
    /// Used to fetch the genesis popow header and, when proving without an
    /// explicit `header_id`, to resolve the suffix head.
    fn popow_header_at_height(&self, height: u32) -> Option<PoPowHeader>;

    /// Returns up to `k` headers at the chain tip in ascending-height
    /// order. Called only when the caller does not specify an explicit
    /// `header_id_opt` to [`crate::NipopowAlgos::prove_with_reader`]; the
    /// first element becomes the suffix head and the rest the suffix tail.
    fn last_headers(&self, k: usize) -> Vec<Header>;

    /// Returns up to `n` headers immediately following `header` in
    /// ascending-height order. Used to construct the suffix tail when an
    /// explicit `header_id_opt` is supplied to
    /// [`crate::NipopowAlgos::prove_with_reader`].
    fn best_headers_after(&self, header: &Header, n: usize) -> Vec<Header>;
}
