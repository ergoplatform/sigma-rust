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
}
