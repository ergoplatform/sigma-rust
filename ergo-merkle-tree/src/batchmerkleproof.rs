use crate::NodeSide;
use crate::{concatenate_hashes, prefixed_hash, prefixed_hash2};
#[derive(Debug)]
pub struct BatchMerkleProof {
    indices: Vec<(usize, [u8; 32])>,
    pub proofs: Vec<crate::LevelNode>,
}

impl BatchMerkleProof {
    pub fn new(indices: Vec<(usize, [u8; 32])>, proofs: Vec<crate::LevelNode>) -> Self {
        BatchMerkleProof { indices, proofs }
    }

    pub fn valid(&self, expected_root: &[u8; 32]) -> bool {
        fn validate(a: &[usize], e: &[(usize, [u8; 32])], m: &[crate::LevelNode]) -> Vec<[u8; 32]> {
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
                    e_new.push(*prefixed_hash(1, &concatenate_hashes(&e[i].1, &e[i + 1].1)));
                    i += 2;
                } else {
                    if m_new[0].1 == NodeSide::Left {
                        e_new.push(*prefixed_hash2(
                            1,
                            m_new[0].0.as_ref().map(|h| h.as_slice()),
                            e[i].1.as_slice(),
                        ));
                    } else {
                        e_new.push(*prefixed_hash2(
                            1,
                            e[i].1.as_slice(),
                            m_new[0].0.as_ref().map(|h| h.as_slice()),
                        ));
                    }

                    m_new.remove(0);
                    i += 1;
                }
            }
            let mut a_new: Vec<usize> = b.iter().map(|(_, b)| b / 2).collect();
            a_new.sort();
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
}
