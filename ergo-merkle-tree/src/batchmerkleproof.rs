use crate::LevelNode;
use crate::NodeSide;
use crate::{prefixed_hash2, INTERNAL_PREFIX};
use ergo_chain_types::Digest32;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct BatchMerkleProofIndex {
    pub index: usize,
    pub hash: Digest32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(into = "crate::json::BatchMerkleProofJson"),
    serde(try_from = "crate::json::BatchMerkleProofJson")
)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
/// Compact Merkle multiproof. Can be created using [`crate::MerkleTree::proof_by_indices`]
/// Implementation based on https://deepai.org/publication/compact-merkle-multiproofs
pub struct BatchMerkleProof {
    pub(crate) indices: Vec<BatchMerkleProofIndex>,
    pub(crate) proofs: Vec<LevelNode>,
}

impl BatchMerkleProof {
    /// Create a new BatchMerkleProof
    pub fn new(indices: Vec<BatchMerkleProofIndex>, proofs: Vec<crate::LevelNode>) -> Self {
        BatchMerkleProof { indices, proofs }
    }

    /// Generates root hash of proof, and compares it against expected root hash
    pub fn valid(&self, expected_root: &[u8]) -> bool {
        fn validate(
            a: &[usize],
            e: &[BatchMerkleProofIndex],
            m: &[crate::LevelNode],
        ) -> Option<Vec<Digest32>> {
            // For each index in a, take the value of its immediate neighbor, and store each index with its neighbor
            let b: Vec<(usize, usize)> = a
                .iter()
                .map(|i| if i % 2 == 0 { (*i, i + 1) } else { (i - 1, *i) })
                .collect();

            let mut e_new = vec![];
            let mut m_new = m.to_owned();
            // E must always have the same length as B
            if e.len() != b.len() {
                return None;
            }
            let mut i = 0;
            // assign generated hashes to a new E that will be used for next iteration
            while i < b.len() {
                if b.len() > 1 && b.get(i) == b.get(i + 1) {
                    // both indices needed for computing parent hash are part of e
                    e_new.push(prefixed_hash2(
                        INTERNAL_PREFIX,
                        e[i].hash.as_ref(),
                        e[i + 1].hash.as_ref(),
                    ));
                    i += 2;
                } else {
                    // Need an additional hash from m
                    let head = if !m_new.is_empty() {
                        m_new.remove(0)
                    } else {
                        return None;
                    };

                    if head.side == NodeSide::Left {
                        e_new.push(prefixed_hash2(
                            INTERNAL_PREFIX,
                            head.hash.as_ref().map(|h| h.as_ref()),
                            e[i].hash.as_ref(),
                        ));
                    } else {
                        e_new.push(prefixed_hash2(
                            INTERNAL_PREFIX,
                            e[i].hash.as_ref(),
                            head.hash.as_ref().map(|h| h.as_ref()),
                        ));
                    }
                    i += 1;
                }
            }
            let mut a_new: Vec<usize> = b.iter().map(|(_, b)| b / 2).collect(); // Generate indices for parents of current b
            a_new.sort_unstable();
            a_new.dedup();
            // Repeat until root of tree is reached
            if !m_new.is_empty() || e_new.len() > 1 {
                let e: Vec<BatchMerkleProofIndex> = a_new
                    .iter()
                    .copied()
                    .zip(e_new.into_iter())
                    .map(|(index, hash)| BatchMerkleProofIndex { index, hash })
                    .collect();
                e_new = validate(&a_new, &e, &m_new)?;
            }
            Some(e_new)
        }

        let mut e = self.indices.to_owned();
        e.sort_by_key(|BatchMerkleProofIndex { index, .. }| *index);
        let a: Vec<usize> = e
            .iter()
            .map(|BatchMerkleProofIndex { index, .. }| *index)
            .collect();
        match validate(&a, &e, &self.proofs).as_deref() {
            Some([root_hash]) => root_hash.as_ref() == expected_root,
            _ => false,
        }
    }

    /// Returns indices (leaf nodes) that are part of the proof
    pub fn get_indices(&self) -> &[BatchMerkleProofIndex] {
        &self.indices
    }
    /// Returns nodes included in proof to get to root node
    pub fn get_proofs(&self) -> &[LevelNode] {
        &self.proofs
    }
}

use sigma_ser::ScorexSerializable;

// Binary Serialization for BatchMerkleProof. Matches Scala implementation. Since the Scala implementation uses 4-byte ints for length/indices, this method will fail the proof or indexes length is > u32::MAX,
impl ScorexSerializable for BatchMerkleProof {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> sigma_ser::ScorexSerializeResult {
        fn write_u32_be<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
            val: u32,
            w: &mut W,
        ) -> sigma_ser::ScorexSerializeResult {
            for byte in val.to_be_bytes() {
                w.put_u8(byte)?;
            }
            Ok(())
        }
        write_u32_be(u32::try_from(self.indices.len())?, w)?; // for serialization, index length must be at most 4 bytes
        write_u32_be(u32::try_from(self.proofs.len())?, w)?;

        for BatchMerkleProofIndex { index, hash } in &self.indices {
            write_u32_be(u32::try_from(*index)?, w)?;
            w.write_all(hash.as_ref())?;
        }

        for proof in &self.proofs {
            match proof.hash {
                Some(ref hash) => w.write_all(hash.as_ref())?,
                None => w.write_all(&[0; 32])?,
            }
            w.put_u8(proof.side as u8)?;
        }

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        fn read_u32_be<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
            r: &mut R,
        ) -> Result<u32, sigma_ser::ScorexParsingError> {
            let mut bytes = [0u8; 4];
            for byte in bytes.iter_mut() {
                *byte = r.get_u8()?;
            }
            Ok(u32::from_be_bytes(bytes))
        }
        let indices_len = read_u32_be(r)? as usize;
        let proofs_len = read_u32_be(r)? as usize;
        let indices = (0..indices_len)
            .map(|_| {
                let index = read_u32_be(r)? as usize;
                let mut hash = Digest32::zero();
                r.read_exact(&mut hash.0[..])?;
                Ok(BatchMerkleProofIndex { index, hash })
            })
            .collect::<Result<Vec<BatchMerkleProofIndex>, sigma_ser::ScorexParsingError>>()?;

        let proofs = (0..proofs_len)
            .map(|_| {
                let mut hash = Digest32::zero();
                r.read_exact(&mut hash.0[..])?;
                let empty = hash.as_ref().iter().all(|&b| b == 0);
                let side: NodeSide = r.get_u8()?.try_into().map_err(|_| {
                    sigma_ser::ScorexParsingError::ValueOutOfBounds(
                        "Side can only be 0 or 1".into(),
                    )
                })?;

                if empty {
                    Ok(crate::LevelNode::empty_node(side))
                } else {
                    Ok(crate::LevelNode::new(hash, side))
                }
            })
            .collect::<Result<Vec<crate::LevelNode>, sigma_ser::ScorexParsingError>>()?;
        Ok(BatchMerkleProof::new(indices, proofs))
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::panic)]
mod test {
    use crate::batchmerkleproof::{BatchMerkleProof, BatchMerkleProofIndex};
    use proptest::prelude::*;
    use sigma_ser::ScorexSerializable;
    proptest! {
        #[test]
        fn test_batchmerkleproof_serialization_roundtrip(proof in any::<BatchMerkleProof>().prop_filter("Indices > u32::max not allowed", |proof| proof.indices.len() < u32::MAX as usize && proof.indices.iter().all(|BatchMerkleProofIndex {index, ..}| *index < u32::MAX as usize))) {
            let serialized_bytes = proof.scorex_serialize_bytes().unwrap();
            assert_eq!(BatchMerkleProof::scorex_parse_bytes(&serialized_bytes).unwrap(), proof);
            assert_eq!(serialized_bytes.len(), (8 + proof.proofs.len() * 33 + proof.indices.len() * 36));
        }
        #[test]
        fn test_empty_deserialization(bytes in any::<[u8; 2]>()) {
            assert!(BatchMerkleProof::scorex_parse_bytes(&bytes).is_err());
        }
        #[test]
        fn test_invalid_deserialization(bytes in any::<[u8; 9]>()) {
            assert!(BatchMerkleProof::scorex_parse_bytes(&bytes).is_err());
        }

    }
}
