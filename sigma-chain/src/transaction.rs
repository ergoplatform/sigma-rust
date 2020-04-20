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

#[derive(PartialEq, Debug)]
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
        w.put_u32(u32::try_from(distinct_token_ids.len()).unwrap())?;
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

    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoLikeTransaction.scala#L146-L146

        // parse transaction inputs
        let inputs_count = r.get_u16()?;
        let mut inputs = Vec::with_capacity(inputs_count as usize);
        for _ in 0..inputs_count {
            inputs.push(Input::sigma_parse(&mut r)?);
        }

        // parse transaction data inputs
        let data_inputs_count = r.get_u16()?;
        let mut data_inputs = Vec::with_capacity(data_inputs_count as usize);
        for _ in 0..data_inputs_count {
            data_inputs.push(DataInput::sigma_parse(&mut r)?);
        }

        // parse distinct ids of tokens in transaction outputs
        let tokens_count = r.get_u32()?;
        let mut token_ids = IndexSet::with_capacity(tokens_count as usize);
        for _ in 0..tokens_count {
            token_ids.insert(TokenId::sigma_parse(&mut r)?);
        }

        // parse outputs
        let outputs_count = r.get_u16()?;
        let mut outputs = Vec::with_capacity(outputs_count as usize);
        for _ in 0..outputs_count {
            outputs.push(ergo_box::parse_body_with_indexed_digests(
                Some(&token_ids),
                &mut r,
            )?)
        }

        Ok(Transaction {
            inputs,
            data_inputs,
            outputs,
        })
    }
}

#[cfg(test)]
use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

#[cfg(test)]
impl Arbitrary for Transaction {
    type Parameters = ();

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            vec(any::<Input>(), 1..10),
            vec(any::<DataInput>(), 0..10),
            vec(any::<ErgoBoxCandidate>(), 1..10),
        )
            .prop_map(|(inputs, data_inputs, outputs)| Self {
                inputs,
                data_inputs,
                outputs,
            })
            .boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Transaction>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
