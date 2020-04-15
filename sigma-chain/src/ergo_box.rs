use crate::ergo_tree::ErgoTree;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::collections::HashMap;
use std::io;

// TODO: extract BoxId
pub const DIGEST32_SIZE: usize = 32;
pub struct BoxId(pub [u8; DIGEST32_SIZE]);

impl SigmaSerializable for BoxId {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        w.write_all(&self.0)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let mut bytes = [0; DIGEST32_SIZE];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes))
    }
}

pub struct TokenId(Box<[u8; DIGEST32_SIZE]>);

pub struct TokenInfo {
    pub token_id: TokenId,
    pub amount: u64,
}

pub struct NonMandatoryRegisterId(u8);

pub struct ErgoBoxCandidate {
    pub value: u64,
    pub ergo_tree: ErgoTree,
    pub tokens: Vec<TokenInfo>,
    pub additional_registers: HashMap<NonMandatoryRegisterId, Box<[u8]>>,
    pub creation_height: u32,
}
