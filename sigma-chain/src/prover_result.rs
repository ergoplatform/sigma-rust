use crate::context_extension::ContextExtension;

pub struct ProverResult {
    pub proof: Box<[u8]>,
    pub extension: ContextExtension,
}
