use crate::ergo_tree::ErgoTree;
use crate::{token_id::TokenId, token_info::TokenInfo};
use indexmap::IndexSet;
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
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error> {
        serialize_body_with_indexed_digests(self, None, w)
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: R) -> Result<Self, SerializationError> {
        parse_body_with_indexed_digests(None, r)
    }
}

pub fn serialize_body_with_indexed_digests<W: vlq_encode::WriteSigmaVlqExt>(
    b: &ErgoBoxCandidate,
    token_ids_in_tx: Option<&IndexSet<TokenId>>,
    w: W,
) -> Result<(), io::Error> {
    unimplemented!()
}

pub fn parse_body_with_indexed_digests<R: vlq_encode::ReadSigmaVlqExt>(
    digests_in_tx: Option<&IndexSet<TokenId>>,
    r: R,
) -> Result<ErgoBoxCandidate, SerializationError> {
    unimplemented!()
}
