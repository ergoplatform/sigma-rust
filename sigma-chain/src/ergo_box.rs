use crate::ergo_tree::ErgoTree;
use crate::token_info::TokenInfo;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::collections::HashMap;
use std::io;

pub struct NonMandatoryRegisterId(u8);

pub struct ErgoBoxCandidate {
    pub value: u64,
    pub ergo_tree: ErgoTree,
    pub tokens: Vec<TokenInfo>,
    pub additional_registers: HashMap<NonMandatoryRegisterId, Box<[u8]>>,
    pub creation_height: u32,
}

impl SigmaSerializable for ErgoBoxCandidate {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, _: W) -> Result<(), io::Error> {
        unimplemented!();
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: R) -> Result<Self, SerializationError> {
        unimplemented!();
    }
}
