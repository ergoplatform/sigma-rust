const INTERLINK_VECTOR_PREFIX: u8 = 0x01;
/// Extension section of Ergo block. Contains key-value storage.
#[derive(Clone, Debug, Default)]
pub struct ExtensionCandidate {
    /// Fields as a sequence of key -> value records. A key is 2-bytes long, value is 64 bytes max.
    pub(crate) fields: Vec<([u8; 2], Vec<u8>)>,
}

impl ExtensionCandidate {
    /// Creates a new [`ExtensionCandidate`] from fields. Fails if a field has a value > 64 bytes
    pub fn new(fields: Vec<([u8; 2], Vec<u8>)>) -> Result<ExtensionCandidate, &'static str> {
        match fields.iter().all(|(_, v)| v.len() <= 64) {
            true => Ok(ExtensionCandidate { fields }),
            false => Err("Values of fields must be less than 64 bytes in size"),
        }
    }
    /// Return fields for this ExtensionCandidate
    pub fn fields(&self) -> &[([u8; 2], Vec<u8>)] {
        &self.fields
    }
    /// Returns fields for this ExtensionCandidate
    pub fn fields_mut(&mut self) -> &mut Vec<([u8; 2], Vec<u8>)> {
        &mut self.fields
    }
    // TODO: memoize merkletree in extensioncandidate fields?
    fn merkletree(&self) -> ergo_merkle_tree::MerkleTree {
        extension_merkletree(&self.fields)
    }

    /// returns a MerkleProof for a single key element
    pub fn proof_for(&self, key: [u8; 2]) -> Option<ergo_merkle_tree::MerkleProof> {
        let tree = self.merkletree();
        let kv = self.fields.iter().find(|(k, _)| *k == key)?;
        tree.proof_by_element(&kv_to_leaf(kv))
    }

    /// Returns a [`ergo_merkle_tree::BatchMerkleProof`] (compact multi-proof) for multiple key elements
    pub fn batch_proof_for(&self, keys: &[[u8; 2]]) -> Option<ergo_merkle_tree::BatchMerkleProof> {
        let tree = self.merkletree();
        let indices: Vec<usize> = keys
            .iter()
            .flat_map(|k| self.fields.iter().find(|(key, _)| key == k))
            .map(kv_to_leaf)
            .map(ergo_merkle_tree::MerkleNode::from)
            .flat_map(|node| node.get_hash().copied())
            .flat_map(|hash| tree.get_elements_hash_index().get(&hash).copied())
            .collect();
        tree.proof_by_indices(&indices)
    }
    /// Returns [`ergo_merkle_tree::BatchMerkleProof`] for block interlinks
    pub fn proof_for_interlink_vector(&self) -> Option<ergo_merkle_tree::BatchMerkleProof> {
        let interlinks: Vec<[u8; 2]> = self
            .fields
            .iter()
            .map(|(key, _)| *key)
            .filter(|key| key[0] == INTERLINK_VECTOR_PREFIX)
            .collect();
        if interlinks.is_empty() {
            Some(ergo_merkle_tree::BatchMerkleProof::new(vec![], vec![]))
        } else {
            self.batch_proof_for(&interlinks)
        }
    }
}

// converts a key value pair to an array of [key.length, key, val]
fn kv_to_leaf(kv: &([u8; 2], Vec<u8>)) -> Vec<u8> {
    std::iter::once(2u8)
        .chain(kv.0.iter().copied())
        .chain(kv.1.iter().copied())
        .collect()
}
// creates a MerkleTree from a key/value pair of extension section
fn extension_merkletree(kv: &[([u8; 2], Vec<u8>)]) -> ergo_merkle_tree::MerkleTree {
    let leafs = kv
        .iter()
        .map(kv_to_leaf)
        .map(ergo_merkle_tree::MerkleNode::from)
        .collect::<Vec<ergo_merkle_tree::MerkleNode>>();
    ergo_merkle_tree::MerkleTree::new(&leafs)
}
