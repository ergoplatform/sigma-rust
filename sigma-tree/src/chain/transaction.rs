//! Ergo transaction

#[cfg(feature = "with-serde")]
use super::json;
use super::{
    data_input::DataInput,
    digest32::{blake2b256_hash, Digest32},
    ergo_box::ErgoBoxCandidate,
    input::Input,
    token::TokenId,
    ErgoBox,
};
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SerializationError,
    SigmaSerializable,
};
use indexmap::IndexSet;
#[cfg(test)]
use proptest_derive::Arbitrary;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use std::io;
use std::iter::FromIterator;
#[cfg(feature = "with-serde")]
use thiserror::Error;

/// Transaction id (ModifierId in sigmastate)
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct TxId(pub Digest32);

impl TxId {
    /// All zeros
    pub fn zero() -> TxId {
        TxId(Digest32::zero())
    }
}

impl SigmaSerializable for TxId {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.0.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self(Digest32::sigma_parse(r)?))
    }
}

#[cfg(feature = "with-serde")]
impl Into<String> for TxId {
    fn into(self) -> String {
        self.0.into()
    }
}

/**
 * ErgoTransaction is an atomic state transition operation. It destroys Boxes from the state
 * and creates new ones. If transaction is spending boxes protected by some non-trivial scripts,
 * its inputs should also contain proof of spending correctness - context extension (user-defined
 * key-value map) and data inputs (links to existing boxes in the state) that may be used during
 * script reduction to crypto, signatures that satisfies the remaining cryptographic protection
 * of the script.
 * Transactions are not encrypted, so it is possible to browse and view every transaction ever
 * collected into a block.
 */
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "with-serde",
    serde(
        try_from = "json::transaction::TransactionJson",
        into = "json::transaction::TransactionJson"
    )
)]
#[derive(PartialEq, Debug, Clone)]
pub struct Transaction {
    tx_id: TxId,
    /// inputs, that will be spent by this transaction.
    pub inputs: Vec<Input>,
    /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
    /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
    /// included in transaction cost and they do not contain spending proofs.
    pub data_inputs: Vec<DataInput>,
    /// box candidates to be created by this transaction. Differ from ordinary ones in that
    /// they do not include transaction id and index
    pub output_candidates: Vec<ErgoBoxCandidate>,
}

impl Transaction {
    /// Creates new transation
    pub fn new(
        inputs: Vec<Input>,
        data_inputs: Vec<DataInput>,
        output_candidates: Vec<ErgoBoxCandidate>,
    ) -> Transaction {
        let tx_to_sign = Transaction {
            tx_id: TxId::zero(),
            inputs,
            data_inputs,
            output_candidates,
        };
        let tx_id = tx_to_sign.calc_tx_id();
        Transaction {
            tx_id,
            ..tx_to_sign
        }
    }

    /// create ErgoBox from ErgoBoxCandidate with tx id and indices
    pub fn outputs(&self) -> Vec<ErgoBox> {
        assert!(self.output_candidates.len() < u16::MAX as usize);
        self.output_candidates
            .iter()
            .enumerate()
            .map(|(idx, bc)| ErgoBox::from_box_candidate(bc, self.tx_id.clone(), idx as u16))
            .collect()
    }

    fn calc_tx_id(&self) -> TxId {
        let bytes = self.bytes_to_sign();
        TxId(blake2b256_hash(&bytes))
    }

    /// Serialized tx with empty proofs
    pub fn bytes_to_sign(&self) -> Vec<u8> {
        let empty_proof_inputs = self.inputs.iter().map(|i| i.input_to_sign()).collect();
        let tx_to_sign = Transaction {
            inputs: empty_proof_inputs,
            ..(*self).clone()
        };
        tx_to_sign.sigma_serialise_bytes()
    }
}

impl SigmaSerializable for Transaction {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoLikeTransaction.scala#L112-L112
        w.put_usize_as_u16(self.inputs.len())?;
        self.inputs.iter().try_for_each(|i| i.sigma_serialize(w))?;
        w.put_usize_as_u16(self.data_inputs.len())?;
        self.data_inputs
            .iter()
            .try_for_each(|i| i.sigma_serialize(w))?;

        // Serialize distinct ids of tokens in transaction outputs.
        // This optimization is crucial to allow up to MaxTokens (== 255) in a box.
        // Without it total size of all token ids 255 * 32 = 8160, way beyond MaxBoxSize (== 4K)
        let token_ids: Vec<TokenId> = self
            .output_candidates
            .iter()
            .flat_map(|b| b.tokens.iter().map(|t| t.token_id.clone()))
            .collect();
        let distinct_token_ids: IndexSet<TokenId> = IndexSet::from_iter(token_ids);
        w.put_u32(u32::try_from(distinct_token_ids.len()).unwrap())?;
        distinct_token_ids
            .iter()
            .try_for_each(|t_id| t_id.sigma_serialize(w))?;

        // serialize outputs
        w.put_usize_as_u16(self.output_candidates.len())?;
        self.output_candidates.iter().try_for_each(|o| {
            ErgoBoxCandidate::serialize_body_with_indexed_digests(o, Some(&distinct_token_ids), w)
        })?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoLikeTransaction.scala#L146-L146

        // parse transaction inputs
        let inputs_count = r.get_u16()?;
        let mut inputs = Vec::with_capacity(inputs_count as usize);
        for _ in 0..inputs_count {
            inputs.push(Input::sigma_parse(r)?);
        }

        // parse transaction data inputs
        let data_inputs_count = r.get_u16()?;
        let mut data_inputs = Vec::with_capacity(data_inputs_count as usize);
        for _ in 0..data_inputs_count {
            data_inputs.push(DataInput::sigma_parse(r)?);
        }

        // parse distinct ids of tokens in transaction outputs
        let tokens_count = r.get_u32()?;
        let mut token_ids = IndexSet::with_capacity(tokens_count as usize);
        for _ in 0..tokens_count {
            token_ids.insert(TokenId::sigma_parse(r)?);
        }

        // parse outputs
        let outputs_count = r.get_u16()?;
        let mut outputs = Vec::with_capacity(outputs_count as usize);
        for _ in 0..outputs_count {
            outputs.push(ErgoBoxCandidate::parse_body_with_indexed_digests(
                Some(&token_ids),
                r,
            )?)
        }

        Ok(Transaction::new(inputs, data_inputs, outputs))
    }
}

#[cfg(feature = "with-serde")]
impl Into<json::transaction::TransactionJson> for Transaction {
    fn into(self) -> json::transaction::TransactionJson {
        json::transaction::TransactionJson {
            tx_id: self.tx_id.clone(),
            inputs: self.inputs.clone(),
            data_inputs: self.data_inputs.clone(),
            outputs: self.outputs(),
        }
    }
}

/// Errors on parsing Transaction from JSON
#[cfg(feature = "with-serde")]
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum TransactionFromJsonError {
    /// Tx id parsed from JSON differs from calculated from serialized bytes
    #[error("Tx id parsed from JSON differs from calculated from serialized bytes")]
    InvalidTxId,
}

#[cfg(feature = "with-serde")]
impl TryFrom<json::transaction::TransactionJson> for Transaction {
    type Error = TransactionFromJsonError;
    fn try_from(tx_json: json::transaction::TransactionJson) -> Result<Self, Self::Error> {
        let output_candidates = tx_json.outputs.iter().map(|o| o.clone().into()).collect();
        let tx_to_sign = Transaction {
            tx_id: TxId::zero(),
            inputs: tx_json.inputs,
            data_inputs: tx_json.data_inputs,
            output_candidates,
        };
        let tx_id = tx_to_sign.calc_tx_id();
        let tx = Transaction {
            tx_id,
            ..tx_to_sign
        };
        if tx.tx_id == tx_json.tx_id {
            Ok(tx)
        } else {
            Err(TransactionFromJsonError::InvalidTxId)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for Transaction {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                vec(any::<Input>(), 1..10),
                vec(any::<DataInput>(), 0..10),
                vec(any::<ErgoBoxCandidate>(), 1..10),
            )
                .prop_map(|(inputs, data_inputs, outputs)| Self::new(inputs, data_inputs, outputs))
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    proptest! {

        #[test]
        fn tx_ser_roundtrip(v in any::<Transaction>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }


        #[test]
        fn tx_id_ser_roundtrip(v in any::<TxId>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }

    #[test]
    #[cfg(feature = "with-serde")]
    fn test_tx_id_calc() {
        let json = r#"
        {
      "id": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
      "inputs": [
        {
          "boxId": "9126af0675056b80d1fda7af9bf658464dbfa0b128afca7bf7dae18c27fe8456",
          "spendingProof": {
            "proofBytes": [],
            "extension": {"values": {}}
          }
        }
      ],
      "dataInputs": [],
      "outputs": [
        {
          "boxId": "b979c439dc698ce5e823b21c722a6e23721af010e4df8c72de0bfd0c3d9ccf6b",
          "value": 74187765000000000,
          "ergoTree": "101004020e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a7017300730110010204020404040004c0fd4f05808c82f5f6030580b8c9e5ae040580f882ad16040204c0944004c0f407040004000580f882ad16d19683030191a38cc7a7019683020193c2b2a57300007473017302830108cdeeac93a38cc7b2a573030001978302019683040193b1a5730493c2a7c2b2a573050093958fa3730673079973089c73097e9a730a9d99a3730b730c0599c1a7c1b2a5730d00938cc7b2a5730e0001a390c1a7730f",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 0
        },
        {
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }
      ]
    }"#;
        let res = serde_json::from_str(json);
        let t: Transaction = res.unwrap();
        let tx_id_str: String = t.tx_id.into();
        assert_eq!(
            "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
            tx_id_str
        )
    }
}
