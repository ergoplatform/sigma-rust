use crate::data_input::DataInput;
use crate::ergo_box::{self, ErgoBoxCandidate};
use crate::{input::Input, token_id::TokenId};
use indexmap::IndexSet;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::convert::TryFrom;
use std::io;
use std::iter::FromIterator;

pub struct Transaction {
    pub inputs: Vec<Input>,
    pub data_inputs: Vec<DataInput>,
    pub outputs: Vec<ErgoBoxCandidate>,
}

impl SigmaSerializable for Transaction {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoLikeTransaction.scala#L112-L112
        w.put_usize_as_u16(self.inputs.len())?;
        self.inputs
            .iter()
            .try_for_each(|i| i.sigma_serialize(&mut w))?;
        self.data_inputs
            .iter()
            .try_for_each(|i| i.sigma_serialize(&mut w))?;

        // Serialize distinct ids of tokens in transaction outputs.
        // This optimization is crucial to allow up to MaxTokens (== 255) in a box.
        // Without it total size of all token ids 255 * 32 = 8160, way beyond MaxBoxSize (== 4K)
        let token_ids: Vec<TokenId> = self
            .outputs
            .iter()
            .flat_map(|b| b.tokens.iter().map(|t| t.token_id))
            .collect();
        let distinct_token_ids: IndexSet<TokenId> = IndexSet::from_iter(token_ids);
        w.put_usize_as_u16(distinct_token_ids.len())?;
        distinct_token_ids
            .iter()
            .try_for_each(|t_id| t_id.sigma_serialize(&mut w))?;

        // serialize outputs
        w.put_usize_as_u16(self.outputs.len())?;
        self.outputs.iter().try_for_each(|o| {
            ergo_box::serialize_body_with_indexed_digests(o, Some(&distinct_token_ids), &mut w)
        })?;
        Ok(())
    }

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: R) -> Result<Self, SerializationError> {
        unimplemented!()
    }
}
