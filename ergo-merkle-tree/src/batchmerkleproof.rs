use crate::LevelNode;
use crate::NodeSide;
use crate::{concatenate_hashes, prefixed_hash, prefixed_hash2};
use crate::{HASH_SIZE, INTERNAL_PREFIX};
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(into = "crate::json::BatchMerkleProofJson"),
    serde(try_from = "crate::json::BatchMerkleProofJson")
)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct BatchMerkleProof {
    pub(crate) indices: Vec<(usize, [u8; 32])>,
    pub(crate) proofs: Vec<LevelNode>,
}

impl BatchMerkleProof {
    pub fn new(indices: Vec<(usize, [u8; HASH_SIZE])>, proofs: Vec<crate::LevelNode>) -> Self {
        BatchMerkleProof { indices, proofs }
    }

    pub fn valid(&self, expected_root: &[u8; HASH_SIZE]) -> bool {
        fn validate(
            a: &[usize],
            e: &[(usize, [u8; HASH_SIZE])],
            m: &[crate::LevelNode],
        ) -> Vec<[u8; HASH_SIZE]> {
            // For each index in a, take the value of its immediate neighbor, and store each index with its neighbor
            let b: Vec<(usize, usize)> = a
                .iter()
                .map(|i| if i % 2 == 0 { (*i, i + 1) } else { (i - 1, *i) })
                .collect();

            let mut e_new = vec![];
            let mut m_new = m.to_owned();

            assert!(e.len() == b.len());
            let mut i = 0;
            while i < b.len() {
                if b.len() > 1 && b.get(i) == b.get(i + 1) {
                    e_new.push(*prefixed_hash(
                        INTERNAL_PREFIX,
                        &concatenate_hashes(&e[i].1, &e[i + 1].1),
                    ));
                    i += 2;
                } else {
                    if m_new[0].1 == NodeSide::Left {
                        e_new.push(*prefixed_hash2(
                            INTERNAL_PREFIX,
                            m_new[0].0.as_ref().map(|h| h.as_slice()),
                            e[i].1.as_slice(),
                        ));
                    } else {
                        e_new.push(*prefixed_hash2(
                            INTERNAL_PREFIX,
                            e[i].1.as_slice(),
                            m_new[0].0.as_ref().map(|h| h.as_slice()),
                        ));
                    }

                    m_new.remove(0);
                    i += 1;
                }
            }
            let mut a_new: Vec<usize> = b.iter().map(|(_, b)| b / 2).collect();
            a_new.sort_unstable();
            a_new.dedup();

            if !m_new.is_empty() || e_new.len() > 1 {
                let e: Vec<(usize, [u8; 32])> =
                    a_new.iter().copied().zip(e_new.into_iter()).collect();
                e_new = validate(&a_new, &e, &m_new);
            }
            e_new
        }

        let mut e = self.indices.to_owned();
        e.sort_by_key(|(index, _)| *index);
        let a: Vec<usize> = e.iter().map(|(index, _)| *index).collect(); // todo
        match &*validate(&a, &e, &self.proofs) {
            [root_hash] => root_hash == expected_root,
            _ => false,
        }
    }

    pub fn get_indices(&self) -> &[(usize, [u8; 32])] {
        &self.indices
    }
    pub fn get_proofs(&self) -> &[LevelNode] {
        &self.proofs
    }
}

use sigma_ser::ScorexSerializable;

// Binary Serialization for BatchMerkleProof. Matches Scala implementation. Since the Scala implementation uses 4-byte ints for length/indices, this method will fail the proof or indexes length is > u32::MAX,
// TODO: use BoundedVec in BatchMerkleProof instead?
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

        for (index, hash) in &self.indices {
            write_u32_be(u32::try_from(*index)?, w)?;
            w.write_all(&hash[..])?;
        }

        for proof in &self.proofs {
            match proof.0 {
                Some(hash) => w.write_all(&hash[..])?,
                None => w.write_all(&[0; 32])?,
            }
            w.put_u8(proof.1 as u8)?;
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
                let mut hash = [0u8; HASH_SIZE];
                r.read_exact(&mut hash)?;
                Ok((index, hash))
            })
            .collect::<Result<Vec<(usize, [u8; HASH_SIZE])>, sigma_ser::ScorexParsingError>>()?;

        let proofs = (0..proofs_len)
            .map(|_| {
                let mut hash = [0u8; HASH_SIZE];
                r.read_exact(&mut hash)?;
                let empty = hash.iter().all(|&b| b == 0);
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
    use crate::batchmerkleproof::BatchMerkleProof;
    use proptest::prelude::*;
    use sigma_ser::ScorexSerializable;
    proptest! {
        #[test]
        fn test_batchmerkleproof_serialization_roundtrip(proof in any::<BatchMerkleProof>().prop_filter("Indices > u32::max not allowed", |proof| proof.indices.len() < u32::MAX as usize && proof.indices.iter().all(|(i, _)| *i < u32::MAX as usize))) {
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
