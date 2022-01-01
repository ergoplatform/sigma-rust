use derive_more::From;
use ergotree_ir::chain::{block_id::BlockId, header::Header};
use serde::{Deserialize, Serialize};
use sigma_ser::{
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
    ScorexParsingError, ScorexSerializable, ScorexSerializeResult,
};

use crate::{
    autolykos_pow_scheme::{self},
    nipopow_algos::NipopowAlgos,
};

#[derive(Serialize, Deserialize)]
/// A structure representing NiPoPow proof as a persistent modifier.
pub struct NipopowProof {
    #[serde(skip_serializing, skip_deserializing)]
    popow_algos: NipopowAlgos,
    /// Security parameter (min μ-level superchain length)
    #[serde(rename = "m")]
    m: u32,
    /// Security parameter (min suffix length, >= 1)
    #[serde(rename = "k")]
    k: u32,
    /// Proof prefix headers
    #[serde(rename = "prefix")]
    prefix: Vec<PoPowHeader>,
    /// First header of the suffix
    #[serde(rename = "suffixHead")]
    suffix_head: PoPowHeader,
    /// Tail of the proof suffix headers
    #[serde(rename = "suffixTail")]
    suffix_tail: Vec<Header>,
}

impl NipopowProof {
    /// Create new proof instance
    pub fn new(
        m: u32,
        k: u32,
        prefix: Vec<PoPowHeader>,
        suffix_head: PoPowHeader,
        suffix_tail: Vec<Header>,
    ) -> NipopowProof {
        NipopowProof {
            popow_algos: NipopowAlgos::default(),
            m,
            k,
            prefix,
            suffix_head,
            suffix_tail,
        }
    }

    /// Implementation of the ≥ algorithm from [`KMZ17`], see Algorithm 4
    ///
    /// [`KMZ17`]: https://fc20.ifca.ai/preproceedings/74.pdf
    pub fn is_better_than(&self, that: NipopowProof) -> bool {
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
                self.popow_algos.best_arg(&self_headers, self.m)
                    > self.popow_algos.best_arg(&that_headers, self.m)
            } else {
                false
            }
        } else {
            self.is_valid()
        }
    }

    fn is_valid(&self) -> bool {
        self.has_valid_connections() && self.has_valid_heights()
    }

    /// Checks the connections of the blocks in the proof. Adjacent blocks should be linked either
    /// via interlink or parent block id. Returns true if all adjacent blocks are correctly
    /// connected.
    fn has_valid_connections(&self) -> bool {
        self.prefix
            .iter()
            .zip(
                self.prefix
                    .iter()
                    .skip(1)
                    .chain(std::iter::once(&self.suffix_head)),
            )
            .all(|(prev, next)| next.interlinks.contains(&prev.header.id))
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

    fn headers_chain(&self) -> impl Iterator<Item = &Header> {
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
        todo!()
        //for h in &self.suffix_tail {
        //    let header_num_bytes = h.scorex_serialize_bytes()?.len();
        //}
        //Ok(())
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(_r: &mut R) -> Result<Self, ScorexParsingError> {
        todo!()
    }
}

/// `NipopowProof` errors
#[derive(PartialEq, Debug, Clone, From)]
pub enum NipopowProofError {
    /// Errors from `AutolykosPowScheme`
    AutolykosPowSchemeError(autolykos_pow_scheme::AutolykosPowSchemeError),
}

#[derive(Serialize, Deserialize)]
/// Stub type until issue #489 is closed.
pub struct PoPowHeader {
    /// The block header
    header: Header,
    /// Interlinks are stored in reverse order: first element is always genesis header, then level
    /// of lowest target met etc
    interlinks: Vec<BlockId>,
}

impl ScorexSerializable for PoPowHeader {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, _w: &mut W) -> ScorexSerializeResult {
        todo!()
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(_r: &mut R) -> Result<Self, ScorexParsingError> {
        todo!()
    }
}
