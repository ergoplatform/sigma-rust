//! Ergo box

mod box_id;
pub mod box_value;
mod register;

use crate::ergo_tree::ErgoTree;
use crate::mir::constant::Constant;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializationError;
use crate::serialization::SigmaSerializeResult;
use crate::util::AsVecI8;

pub use box_id::*;
pub use register::*;

use indexmap::IndexSet;
use sigma_util::hash::blake2b256_hash;
use std::convert::TryFrom;

use std::convert::TryInto;

use self::box_value::BoxValue;

use super::digest32::Digest32;
use super::token::Token;
use super::token::TokenId;
use super::tx_id::TxId;

/// Box (aka coin, or an unspent output) is a basic concept of a UTXO-based cryptocurrency.
/// In Bitcoin, such an object is associated with some monetary value (arbitrary,
/// but with predefined precision, so we use integer arithmetic to work with the value),
/// and also a guarding script (aka proposition) to protect the box from unauthorized opening.
///
/// In other way, a box is a state element locked by some proposition (ErgoTree).
///
/// In Ergo, box is just a collection of registers, some with mandatory types and semantics,
/// others could be used by applications in any way.
/// We add additional fields in addition to amount and proposition~(which stored in the registers R0 and R1).
/// Namely, register R2 contains additional tokens (a sequence of pairs (token identifier, value)).
/// Register R3 contains height specified by user (protocol checks if it was <= current height when
/// transaction was accepted) and also transaction identifier and box index in the transaction outputs.
/// Registers R4-R9 are free for arbitrary usage.
///
/// A transaction is unsealing a box. As a box can not be open twice, any further valid transaction
/// can not be linked to the same box.
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(try_from = "crate::chain::json::ergo_box::ErgoBoxFromJson")
)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBox {
    #[cfg_attr(feature = "json", serde(rename = "boxId", alias = "id"))]
    box_id: BoxId,
    /// amount of money associated with the box
    #[cfg_attr(feature = "json", serde(rename = "value"))]
    pub value: BoxValue,
    /// guarding script, which should be evaluated to true in order to open this box
    #[cfg_attr(
        feature = "json",
        serde(rename = "ergoTree", with = "crate::chain::json::ergo_tree")
    )]
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    #[cfg_attr(feature = "json", serde(rename = "assets"))]
    pub tokens: Vec<Token>,
    ///  additional registers the box can carry over
    #[cfg_attr(feature = "json", serde(rename = "additionalRegisters"))]
    pub additional_registers: NonMandatoryRegisters,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    #[cfg_attr(feature = "json", serde(rename = "creationHeight"))]
    pub creation_height: u32,
    /// id of transaction which created the box
    #[cfg_attr(feature = "json", serde(rename = "transactionId", alias = "txId"))]
    pub transaction_id: TxId,
    /// number of box (from 0 to total number of boxes the transaction with transactionId created - 1)
    #[cfg_attr(feature = "json", serde(rename = "index"))]
    pub index: u16,
}

impl ErgoBox {
    /// Maximum number of tokens in the box
    pub const MAX_TOKENS_COUNT: usize = u8::MAX as usize;

    /// Crate new box
    pub fn new(
        value: BoxValue,
        ergo_tree: ErgoTree,
        tokens: Vec<Token>,
        additional_registers: NonMandatoryRegisters,
        creation_height: u32,
        transaction_id: TxId,
        index: u16,
    ) -> Result<ErgoBox, SigmaSerializationError> {
        let box_with_zero_id = ErgoBox {
            box_id: BoxId::zero(),
            value,
            ergo_tree,
            tokens,
            additional_registers,
            creation_height,
            transaction_id,
            index,
        };
        let box_id = box_with_zero_id.calc_box_id()?;
        Ok(ErgoBox {
            box_id,
            ..box_with_zero_id
        })
    }

    /// Box id (Blake2b256 hash of serialized box)
    pub fn box_id(&self) -> BoxId {
        self.box_id.clone()
    }

    /// Create ErgoBox from ErgoBoxCandidate by adding transaction id
    /// and index of the box in the transaction
    pub fn from_box_candidate(
        box_candidate: &ErgoBoxCandidate,
        transaction_id: TxId,
        index: u16,
    ) -> Result<ErgoBox, SigmaSerializationError> {
        let box_with_zero_id = ErgoBox {
            box_id: BoxId::zero(),
            value: box_candidate.value,
            ergo_tree: box_candidate.ergo_tree.clone(),
            tokens: box_candidate.tokens.clone(),
            additional_registers: box_candidate.additional_registers.clone(),
            creation_height: box_candidate.creation_height,
            transaction_id,
            index,
        };
        let box_id = box_with_zero_id.calc_box_id()?;
        Ok(ErgoBox {
            box_id,
            ..box_with_zero_id
        })
    }

    pub(crate) fn calc_box_id(&self) -> Result<BoxId, SigmaSerializationError> {
        let bytes = self.sigma_serialize_bytes()?;
        let hash = blake2b256_hash(&bytes);
        Ok(Digest32::from(*hash).into())
    }

    /// Get register value
    pub fn get_register(&self, id: RegisterId) -> Option<Constant> {
        match id {
            RegisterId::MandatoryRegisterId(id) => match id {
                MandatoryRegisterId::R0 => Some(self.value.into()),
                // chance of box script is not serializable are tiny comparing to returning Result
                #[allow(clippy::unwrap_used)]
                MandatoryRegisterId::R1 => Some(self.script_bytes().unwrap().into()),
                MandatoryRegisterId::R2 => Some(self.tokens_raw().into()),
                MandatoryRegisterId::R3 => Some(self.creation_info().into()),
            },
            RegisterId::NonMandatoryRegisterId(id) => self.additional_registers.get(id).cloned(),
        }
    }

    /// Returns tokens as tuple of byte array and amount as primitive types
    pub fn tokens_raw(&self) -> Vec<(Vec<i8>, i64)> {
        self.tokens.clone().into_iter().map(Into::into).collect()
    }

    /// Returns serialized ergo_tree guarding this box
    pub fn script_bytes(&self) -> Result<Vec<i8>, SigmaSerializationError> {
        Ok(self.ergo_tree.sigma_serialize_bytes()?.as_vec_i8())
    }

    /// Tuple of height when block got included into the blockchain and transaction identifier with
    /// box index in the transaction outputs serialized to the byte array.
    pub fn creation_info(&self) -> (i32, Vec<i8>) {
        let mut bytes = Vec::with_capacity(Digest32::SIZE + 2);
        bytes.extend_from_slice(self.transaction_id.0 .0.as_ref());
        bytes.extend_from_slice(&self.index.to_be_bytes());
        (self.creation_height as i32, bytes.as_vec_i8())
    }

    fn bytes_without_ref(&self) -> Result<Vec<i8>, SigmaSerializationError> {
        let candidate: ErgoBoxCandidate = self.clone().into();
        Ok(candidate.sigma_serialize_bytes()?.as_vec_i8())
    }
}

impl SigmaSerializable for ErgoBox {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        let ergo_tree_bytes = self.ergo_tree.sigma_serialize_bytes()?;
        serialize_box_with_indexed_digests(
            &self.value,
            ergo_tree_bytes,
            &self.tokens,
            &self.additional_registers,
            self.creation_height,
            None,
            w,
        )?;
        self.transaction_id.sigma_serialize(w)?;
        w.put_u16(self.index)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let box_candidate = ErgoBoxCandidate::parse_body_with_indexed_digests(None, r)?;
        let tx_id = TxId::sigma_parse(r)?;
        let index = r.get_u16()?;
        Ok(ErgoBox::from_box_candidate(&box_candidate, tx_id, index)?)
    }
}

#[cfg(feature = "json")]
impl TryFrom<super::json::ergo_box::ErgoBoxFromJson> for ErgoBox {
    type Error = super::json::ergo_box::ErgoBoxFromJsonError;
    fn try_from(box_json: super::json::ergo_box::ErgoBoxFromJson) -> Result<Self, Self::Error> {
        let box_with_zero_id = ErgoBox {
            box_id: BoxId::zero(),
            value: box_json.value,
            ergo_tree: box_json.ergo_tree,
            tokens: box_json.tokens,
            additional_registers: box_json.additional_registers,
            creation_height: box_json.creation_height,
            transaction_id: box_json.transaction_id,
            index: box_json.index,
        };
        let box_id = box_with_zero_id.calc_box_id()?;
        let ergo_box = ErgoBox {
            box_id,
            ..box_with_zero_id
        };
        match box_json.box_id {
            Some(box_id) => {
                if ergo_box.box_id() == box_id {
                    Ok(ergo_box)
                } else {
                    Err(super::json::ergo_box::ErgoBoxFromJsonError::InvalidBoxId)
                }
            }
            None => Ok(ergo_box),
        }
    }
}

/// Contains the same fields as `ErgoBox`, except if transaction id and index,
/// that will be calculated after full transaction formation.
/// Use [`box_builder::ErgoBoxCandidateBuilder`] to create an instance.
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ErgoBoxCandidate {
    /// amount of money associated with the box
    #[cfg_attr(feature = "json", serde(rename = "value"))]
    pub value: BoxValue,
    /// guarding script, which should be evaluated to true in order to open this box
    #[cfg_attr(
        feature = "json",
        serde(rename = "ergoTree", with = "crate::chain::json::ergo_tree")
    )]
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    #[cfg_attr(feature = "json", serde(rename = "assets"))]
    pub tokens: Vec<Token>,
    ///  additional registers the box can carry over
    #[cfg_attr(feature = "json", serde(rename = "additionalRegisters"))]
    pub additional_registers: NonMandatoryRegisters,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    #[cfg_attr(feature = "json", serde(rename = "creationHeight"))]
    pub creation_height: u32,
}

impl ErgoBoxCandidate {
    /// Box serialization with token ids optionally saved in transaction
    /// (in this case only token index is saved)
    pub fn serialize_body_with_indexed_digests<W: SigmaByteWrite>(
        &self,
        token_ids_in_tx: Option<&IndexSet<TokenId>>,
        w: &mut W,
    ) -> SigmaSerializeResult {
        serialize_box_with_indexed_digests(
            &self.value,
            self.ergo_tree.sigma_serialize_bytes()?,
            &self.tokens,
            &self.additional_registers,
            self.creation_height,
            token_ids_in_tx,
            w,
        )
    }

    /// Box deserialization with token ids optionally parsed in transaction
    pub fn parse_body_with_indexed_digests<R: SigmaByteRead>(
        digests_in_tx: Option<&IndexSet<TokenId>>,
        r: &mut R,
    ) -> Result<ErgoBoxCandidate, SigmaParsingError> {
        parse_box_with_indexed_digests(digests_in_tx, r)
    }
}

impl SigmaSerializable for ErgoBoxCandidate {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.serialize_body_with_indexed_digests(None, w)
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        ErgoBoxCandidate::parse_body_with_indexed_digests(None, r)
    }
}

impl From<ErgoBox> for ErgoBoxCandidate {
    fn from(b: ErgoBox) -> Self {
        ErgoBoxCandidate {
            value: b.value,
            ergo_tree: b.ergo_tree,
            tokens: b.tokens,
            additional_registers: b.additional_registers,
            creation_height: b.creation_height,
        }
    }
}

/// ErgoBox and ErgoBoxCandidate serialization

/// Box serialization with token ids optionally saved in transaction
/// (in this case only token index is saved)
pub fn serialize_box_with_indexed_digests<W: SigmaByteWrite>(
    box_value: &BoxValue,
    ergo_tree_bytes: Vec<u8>,
    tokens: &[Token],
    additional_registers: &NonMandatoryRegisters,
    creation_height: u32,
    token_ids_in_tx: Option<&IndexSet<TokenId>>,
    w: &mut W,
) -> SigmaSerializeResult {
    // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoBoxCandidate.scala#L95-L95
    box_value.sigma_serialize(w)?;
    w.write_all(&ergo_tree_bytes[..])?;
    w.put_u32(creation_height)?;
    // until https://github.com/ergoplatform/sigma-rust/issues/416 is done
    #[allow(clippy::unwrap_used)]
    w.put_u8(u8::try_from(tokens.len()).unwrap())?;

    tokens.iter().try_for_each(|t| {
        match token_ids_in_tx {
            Some(token_ids) => Ok(w.put_u32(
                #[allow(clippy::unwrap_used)]
                u32::try_from(
                    #[allow(clippy::expect_used)]
                    token_ids
                        .get_full(&t.token_id)
                        // this is not a true runtime error it just means that
                        // calling site messed up the token ids
                        .expect("failed to find token id in tx's digest index")
                        .0,
                )
                .unwrap(),
            )?),
            None => t.token_id.sigma_serialize(w),
        }
        .and_then(|()| Ok(w.put_u64(t.amount.into())?))
    })?;
    additional_registers.sigma_serialize(w)
}

/// Box deserialization with token ids optionally parsed in transaction
pub fn parse_box_with_indexed_digests<R: SigmaByteRead>(
    digests_in_tx: Option<&IndexSet<TokenId>>,
    r: &mut R,
) -> Result<ErgoBoxCandidate, SigmaParsingError> {
    // reference implementation -https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoBoxCandidate.scala#L144-L144

    let value = BoxValue::sigma_parse(r)?;
    let ergo_tree = ErgoTree::sigma_parse(r)?;
    let creation_height = r.get_u32()?;
    let tokens_count = r.get_u8()?;
    let mut tokens = Vec::with_capacity(tokens_count as usize);
    for _ in 0..tokens_count {
        let token_id = match digests_in_tx {
            None => TokenId::sigma_parse(r)?,
            Some(digests) => {
                let digest_index = r.get_u32()?;
                match digests.get_index(digest_index as usize) {
                    Some(i) => Ok((*i).clone()),
                    None => Err(SigmaParsingError::Misc(
                        "failed to find token id in tx digests".to_string(),
                    )),
                }?
            }
        };
        let amount = r.get_u64()?;
        tokens.push(Token {
            token_id,
            amount: amount.try_into()?,
        })
    }

    let additional_registers = NonMandatoryRegisters::sigma_parse(r)?;

    Ok(ErgoBoxCandidate {
        value,
        ergo_tree,
        tokens,
        additional_registers,
        creation_height,
    })
}

/// Arbitrary
#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::box_value::arbitrary::ArbBoxValueRange;
    use super::*;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for ErgoBoxCandidate {
        type Parameters = ArbBoxValueRange;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<BoxValue>(args),
                any::<ErgoTree>(),
                vec(any::<Token>(), 0..3),
                any::<u32>(),
                any::<NonMandatoryRegisters>(),
            )
                .prop_map(
                    |(value, ergo_tree, tokens, creation_height, additional_registers)| Self {
                        value,
                        ergo_tree,
                        tokens,
                        additional_registers,
                        creation_height,
                    },
                )
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    impl Arbitrary for ErgoBox {
        type Parameters = ArbBoxValueRange;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<ErgoBoxCandidate>(args),
                any::<TxId>(),
                any::<u16>(),
            )
                .prop_map(|(box_candidate, tx_id, index)| {
                    Self::from_box_candidate(&box_candidate, tx_id, index).unwrap()
                })
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    impl ErgoBox {
        /// Returns copy of the current ErgoBox with given additional registers set
        pub fn with_additional_registers(self, registers: NonMandatoryRegisters) -> ErgoBox {
            ErgoBox {
                additional_registers: registers,
                ..self
            }
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    #[test]
    fn get_register_mandatory() {
        let b = force_any_val::<ErgoBox>();
        assert_eq!(b.get_register(RegisterId::R0).unwrap(), b.value.into());
        assert_eq!(
            b.get_register(RegisterId::R1).unwrap(),
            b.script_bytes().unwrap().into()
        );
        assert_eq!(
            b.get_register(RegisterId::R2).unwrap(),
            b.tokens_raw().into()
        );
        assert_eq!(
            b.get_register(RegisterId::R3).unwrap(),
            b.creation_info().into()
        );
    }

    #[test]
    fn creation_info() {
        let b = force_any_val::<ErgoBox>();
        assert_eq!(b.creation_info().0, b.creation_height as i32);
        let mut expected_bytes = Vec::new();
        expected_bytes.extend_from_slice(b.transaction_id.0 .0.as_ref());
        expected_bytes.extend_from_slice(&b.index.to_be_bytes());
        assert_eq!(b.creation_info().1, expected_bytes.to_vec().as_vec_i8());
    }

    proptest! {

        #[test]
        fn ergo_box_candidate_ser_roundtrip(v in any::<ErgoBoxCandidate>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }

        #[test]
        fn ergo_box_ser_roundtrip(v in any::<ErgoBox>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
