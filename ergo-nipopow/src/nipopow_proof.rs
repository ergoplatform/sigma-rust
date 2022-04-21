use derive_more::From;
use ergo_chain_types::BlockId;
use ergo_merkle_tree::BatchMerkleProof;
use ergotree_ir::chain::header::Header;
use serde::{Deserialize, Serialize};
use sigma_ser::{
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
    ScorexParsingError, ScorexSerializable, ScorexSerializeResult,
};
use std::convert::TryFrom;

use crate::{autolykos_pow_scheme, nipopow_algos::NipopowAlgos};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A structure representing NiPoPow proof as a persistent modifier.
pub struct NipopowProof {
    /// Algos
    #[serde(skip_serializing, skip_deserializing)]
    pub popow_algos: NipopowAlgos,
    /// Security parameter (min μ-level superchain length)
    #[serde(rename = "m")]
    pub m: u32,
    /// Security parameter (min suffix length, >= 1)
    #[serde(rename = "k")]
    pub k: u32,
    /// Proof prefix headers
    #[serde(rename = "prefix")]
    pub prefix: Vec<PoPowHeader>,
    /// First header of the suffix
    #[serde(rename = "suffixHead")]
    pub suffix_head: PoPowHeader,
    /// Tail of the proof suffix headers
    #[serde(rename = "suffixTail")]
    pub suffix_tail: Vec<Header>,
}

impl NipopowProof {
    /// Create new proof instance
    pub fn new(
        m: u32,
        k: u32,
        prefix: Vec<PoPowHeader>,
        suffix_head: PoPowHeader,
        suffix_tail: Vec<Header>,
    ) -> Result<NipopowProof, NipopowProofError> {
        if k >= 1 {
            Ok(NipopowProof {
                popow_algos: NipopowAlgos::default(),
                m,
                k,
                prefix,
                suffix_head,
                suffix_tail,
            })
        } else {
            Err(NipopowProofError::ZeroKParameter)
        }
    }

    /// Implementation of the ≥ algorithm from [`KMZ17`], see Algorithm 4
    ///
    /// [`KMZ17`]: https://fc20.ifca.ai/preproceedings/74.pdf
    pub fn is_better_than(&self, that: &NipopowProof) -> Result<bool, NipopowProofError> {
        if self.is_valid() && that.is_valid() {
            if let Some(lca) = self.popow_algos.lowest_common_ancestor(
                &self.headers_chain().collect::<Vec<_>>(),
                &that.headers_chain().collect::<Vec<_>>(),
            ) {
                let self_headers = self
                    .headers_chain()
                    .filter(|h| h.height > lca.height)
                    .collect::<Vec<_>>();
                let that_headers = that
                    .headers_chain()
                    .filter(|h| h.height > lca.height)
                    .collect::<Vec<_>>();
                Ok(self.popow_algos.best_arg(&self_headers, self.m)?
                    > self.popow_algos.best_arg(&that_headers, self.m)?)
            } else {
                Ok(false)
            }
        } else {
            Ok(self.is_valid())
        }
    }

    fn is_valid(&self) -> bool {
        self.has_valid_connections() && self.has_valid_heights() && self.has_valid_proofs()
    }

    /// Checks the connections of the blocks in the proof. Adjacent blocks should be linked either
    /// via interlink or parent block id. Returns true if all adjacent blocks are correctly
    /// connected.
    pub fn has_valid_connections(&self) -> bool {
        self.prefix
            .iter()
            .zip(
                self.prefix
                    .iter()
                    .skip(1)
                    .chain(std::iter::once(&self.suffix_head)),
            )
            .all(|(prev, next)| {
                // Note that blocks with level 0 do not appear at all within interlinks, which is
                // why we need to check the parent block id as well.
                next.interlinks.contains(&prev.header.id) || next.header.parent_id == prev.header.id
            })
            && std::iter::once(&self.suffix_head.header)
                .chain(self.suffix_tail.iter())
                .zip(self.suffix_tail.iter())
                .all(|(prev, next)| next.parent_id == prev.id)
    }

    /// Checks if the heights of the header-chain provided are consistent, meaning that for any two
    /// blocks b1 and b2, if b1 precedes b2 then b1's height should be smaller. Return true if the
    /// heights of the header-chain are consistent
    fn has_valid_heights(&self) -> bool {
        self.headers_chain()
            .zip(self.headers_chain().skip(1))
            .all(|(prev, next)| prev.height < next.height)
    }
    /// Checks interlink proofs for each block using `PoPowHeader::check_interlinks_proof`
    fn has_valid_proofs(&self) -> bool {
        std::iter::once(&self.suffix_head)
            .chain(self.prefix.iter())
            .all(PoPowHeader::check_interlinks_proof)
    }

    /// Returns an iterator representing a chain of `Headers` from `self.prefix`, to
    /// `self.suffix_head` and `self.suffix_tail`.
    pub(crate) fn headers_chain(&self) -> impl Iterator<Item = &Header> {
        self.prefix
            .iter()
            .map(|p| &p.header)
            .chain(std::iter::once(&self.suffix_head.header).chain(self.suffix_tail.iter()))
    }
}

impl ScorexSerializable for NipopowProof {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        w.put_u32(self.m)?;
        w.put_u32(self.k)?;
        w.put_u32(self.prefix.len() as u32)?;
        for p in &self.prefix {
            let prefix_num_bytes = p.scorex_serialize_bytes()?.len();
            w.put_u32(prefix_num_bytes as u32)?;
            p.scorex_serialize(w)?;
        }
        let suffix_head_num_bytes = self.suffix_head.scorex_serialize_bytes()?.len();
        w.put_u32(suffix_head_num_bytes as u32)?;
        self.suffix_head.scorex_serialize(w)?;
        w.put_u32(self.suffix_tail.len() as u32)?;
        for h in &self.suffix_tail {
            let header_num_bytes = h.scorex_serialize_bytes()?.len();
            w.put_u32(header_num_bytes as u32)?;
            h.scorex_serialize(w)?;
        }
        Ok(())
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let m = r.get_u32()?;
        let k = r.get_u32()?;
        let num_prefixes = r.get_u32()? as usize;
        let mut prefix = Vec::with_capacity(num_prefixes);
        for _ in 0..num_prefixes {
            let _size = r.get_u32()?;
            prefix.push(PoPowHeader::scorex_parse(r)?);
        }
        let _suffix_head_size = r.get_u32()?;
        let suffix_head = PoPowHeader::scorex_parse(r)?;
        let num_suffix_tail = r.get_u32()? as usize;
        let mut suffix_tail = Vec::with_capacity(num_suffix_tail);
        for _ in 0..num_suffix_tail {
            let _size = r.get_u32();
            suffix_tail.push(Header::scorex_parse(r)?);
        }
        Ok(NipopowProof {
            popow_algos: NipopowAlgos::default(),
            m,
            k,
            prefix,
            suffix_head,
            suffix_tail,
        })
    }
}

/// `NipopowProof` errors
#[derive(PartialEq, Debug, Clone, From)]
pub enum NipopowProofError {
    /// Errors from `AutolykosPowScheme`
    AutolykosPowSchemeError(autolykos_pow_scheme::AutolykosPowSchemeError),
    /// `k` parameter == 0. Must be >= 1.
    ZeroKParameter,
    /// Can not prove non-anchored (first block is non-Genesis) chain
    NonAnchoredChain,
    /// Chain must be of length `>= k + m`
    ChainTooShort,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// PoPowHeader structure. Represents the block header and unpacked interlinks
pub struct PoPowHeader {
    /// The block header
    pub header: Header,
    /// Interlinks are stored in reverse order: first element is always genesis header, then level
    /// of lowest target met etc
    pub interlinks: Vec<BlockId>,
    /// BatchMerkleProof for interlinks in extension field
    pub interlinks_proof: BatchMerkleProof,
}

impl PoPowHeader {
    /// Validates interlinks merkle root against provided proof
    pub fn check_interlinks_proof(&self) -> bool {
        if self.interlinks.is_empty()
            && self.interlinks_proof.get_indices().is_empty()
            && self.interlinks_proof.get_proofs().is_empty()
        {
            true
        } else {
            let fields: Vec<ergo_merkle_tree::MerkleNode> =
                NipopowAlgos::pack_interlinks(self.interlinks.clone())
                    .into_iter()
                    .map(|(k, v)| -> Vec<u8> {
                        std::iter::once(2u8)
                            .chain(k.iter().copied())
                            .chain(v.into_iter())
                            .collect()
                    })
                    .map(ergo_merkle_tree::MerkleNode::try_from)
                    .map(Result::unwrap)
                    .collect();
            let tree = ergo_merkle_tree::MerkleTree::new(&fields);
            self.interlinks_proof.valid(tree.root_hash().as_ref())
        }
    }
}

impl ScorexSerializable for PoPowHeader {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        let bytes = self.header.scorex_serialize_bytes()?;
        w.put_u32(bytes.len() as u32)?;
        w.write_all(&bytes)?;
        w.put_u32(self.interlinks.len() as u32)?;
        for interlink in self.interlinks.iter() {
            w.write_all(&*interlink.0 .0)?;
        }
        let proof_bytes = self.interlinks_proof.scorex_serialize_bytes()?;
        w.put_u32(proof_bytes.len() as u32)?;
        w.write_all(&proof_bytes)?;

        Ok(())
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let header_size = r.get_u32()?;
        let mut buf = vec![0; header_size as usize];
        r.read_exact(&mut buf)?;
        let header = Header::scorex_parse(&mut std::io::Cursor::new(buf))?;

        let interlinks_size = r.get_u32()?;

        let interlinks: Result<Vec<BlockId>, ScorexParsingError> = (0..interlinks_size)
            .map(|_| {
                let mut buf = [0; 32];
                r.read_exact(&mut buf)?;
                Ok(BlockId(buf.into()))
            })
            .collect();

        let proof_bytes = r.get_u32()? as usize;
        let mut proof_buf = vec![0u8; proof_bytes];
        r.read_exact(&mut proof_buf)?;
        let interlinks_proof = BatchMerkleProof::scorex_parse_bytes(&proof_buf);

        Ok(Self {
            header,
            interlinks: interlinks?,
            interlinks_proof: interlinks_proof?,
        })
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use crate::autolykos_pow_scheme::AutolykosPowScheme;

    use super::*;
    use ergo_chain_types::Digest32;
    use ergo_chain_types::ExtensionCandidate;
    use proptest::prelude::*;
    use proptest::{arbitrary::Arbitrary, collection::vec};

    impl Arbitrary for PoPowHeader {
        type Parameters = ();
        type Strategy = BoxedStrategy<PoPowHeader>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Box<Header>>(), vec(any::<Digest32>(), 1..10))
                .prop_map(|(header, digests)| PoPowHeader {
                    header: *header,
                    interlinks: digests.iter().cloned().map(BlockId).collect(),
                    interlinks_proof: NipopowAlgos::proof_for_interlink_vector(
                        &ExtensionCandidate::new(NipopowAlgos::pack_interlinks(
                            digests.into_iter().map(BlockId).collect(),
                        ))
                        .unwrap(),
                    )
                    .unwrap(),
                })
                .boxed()
        }
    }

    impl Arbitrary for NipopowProof {
        type Parameters = ();
        type Strategy = BoxedStrategy<NipopowProof>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<u32>(),
                any::<u32>(),
                vec(any::<PoPowHeader>(), 1..10),
                any::<PoPowHeader>(),
                vec(any::<Header>(), 1..10),
            )
                .prop_map(|(m, k, prefix, suffix_head, suffix_tail)| NipopowProof {
                    popow_algos: NipopowAlgos {
                        pow_scheme: AutolykosPowScheme::default(),
                    },
                    m,
                    k,
                    prefix,
                    suffix_head,
                    suffix_tail,
                })
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::panic)]
pub mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::scorex_serialize_roundtrip;
    proptest! {

        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn nipopowproof_roundtrip(v in any::<NipopowProof>()) {
            prop_assert_eq![scorex_serialize_roundtrip(&v), v];
        }


    }
}
