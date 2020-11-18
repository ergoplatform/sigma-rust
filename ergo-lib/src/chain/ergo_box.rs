//! Ergo box

pub mod box_builder;
mod box_id;
mod box_value;
mod register;

pub use box_id::*;
pub use box_value::*;
pub use register::*;

#[cfg(feature = "json")]
use super::json;
use super::token::TokenAmount;
use super::{
    digest32::blake2b256_hash,
    token::{Token, TokenId},
    transaction::TxId,
};

use crate::{
    ergo_tree::ErgoTree,
    serialization::{
        ergo_box::{parse_box_with_indexed_digests, serialize_box_with_indexed_digests},
        sigma_byte_reader::SigmaByteRead,
        sigma_byte_writer::SigmaByteWrite,
        SerializationError, SigmaSerializable,
    },
};
use indexmap::IndexSet;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "json")]
use std::convert::TryFrom;
use std::io;
#[cfg(feature = "json")]
use thiserror::Error;

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
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(try_from = "json::ergo_box::ErgoBoxFromJson"))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBox {
    #[cfg_attr(feature = "json", serde(rename = "boxId"))]
    box_id: BoxId,
    /// amount of money associated with the box
    #[cfg_attr(feature = "json", serde(rename = "value"))]
    pub value: BoxValue,
    /// guarding script, which should be evaluated to true in order to open this box
    #[cfg_attr(feature = "json", serde(rename = "ergoTree", with = "json::ergo_tree"))]
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
    #[cfg_attr(feature = "json", serde(rename = "transactionId"))]
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
    ) -> ErgoBox {
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
        let box_id = box_with_zero_id.calc_box_id();
        ErgoBox {
            box_id,
            ..box_with_zero_id
        }
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
    ) -> ErgoBox {
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
        let box_id = box_with_zero_id.calc_box_id();
        ErgoBox {
            box_id,
            ..box_with_zero_id
        }
    }

    fn calc_box_id(&self) -> BoxId {
        let bytes = self.sigma_serialize_bytes();
        BoxId(blake2b256_hash(&bytes))
    }
}

/// Assets that ErgoBox holds
pub trait ErgoBoxAssets {
    /// Box value
    fn value(&self) -> BoxValue;
    /// Tokens (ids and amounts)
    fn tokens(&self) -> Vec<Token>;
}

/// Returns the total value of the given boxes
pub fn sum_value<T: ErgoBoxAssets>(bs: &[T]) -> u64 {
    bs.iter().map(|b| *b.value().as_u64()).sum()
}

/// Returns the total token amounts (all tokens combined)
pub fn sum_tokens(ts: &[Token]) -> HashMap<TokenId, TokenAmount> {
    let mut res: HashMap<TokenId, TokenAmount> = HashMap::new();
    ts.iter().for_each(|t| {
        res.entry(t.token_id.clone())
            .and_modify(|amt| *amt = amt.checked_add(&t.amount).unwrap())
            .or_insert(t.amount);
    });
    res
}

/// Returns the total token amounts (all tokens combined) of the given boxes
pub fn sum_tokens_from_boxes<T: ErgoBoxAssets>(bs: &[T]) -> HashMap<TokenId, TokenAmount> {
    let mut res: HashMap<TokenId, TokenAmount> = HashMap::new();
    bs.iter().for_each(|b| {
        b.tokens().iter().for_each(|t| {
            res.entry(t.token_id.clone())
                .and_modify(|amt| *amt = amt.checked_add(&t.amount).unwrap())
                .or_insert(t.amount);
        });
    });
    res
}

/// Simple struct to hold ErgoBoxAssets values
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxAssetsData {
    /// Box value
    pub value: BoxValue,
    /// Tokens
    pub tokens: Vec<Token>,
}

impl ErgoBoxAssets for ErgoBoxAssetsData {
    fn value(&self) -> BoxValue {
        self.value
    }

    fn tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }
}

impl ErgoBoxAssets for ErgoBoxCandidate {
    fn value(&self) -> BoxValue {
        self.value
    }

    fn tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }
}

impl ErgoBoxAssets for ErgoBox {
    fn value(&self) -> BoxValue {
        self.value
    }

    fn tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }
}

/// id of the ergo box
pub trait ErgoBoxId {
    /// Id of the ergo box
    fn box_id(&self) -> BoxId;
}

impl ErgoBoxId for ErgoBox {
    fn box_id(&self) -> BoxId {
        self.box_id.clone()
    }
}

/// Errors on parsing ErgoBox from JSON
#[cfg(feature = "json")]
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ErgoBoxFromJsonError {
    /// Box id parsed from JSON differs from calculated from box serialized bytes
    #[error("Box id parsed from JSON differs from calculated from box serialized bytes")]
    InvalidBoxId,
}

#[cfg(feature = "json")]
impl TryFrom<json::ergo_box::ErgoBoxFromJson> for ErgoBox {
    type Error = ErgoBoxFromJsonError;
    fn try_from(box_json: json::ergo_box::ErgoBoxFromJson) -> Result<Self, Self::Error> {
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
        let box_id = box_with_zero_id.calc_box_id();
        let ergo_box = ErgoBox {
            box_id,
            ..box_with_zero_id
        };
        if ergo_box.box_id() == box_json.box_id {
            Ok(ergo_box)
        } else {
            Err(ErgoBoxFromJsonError::InvalidBoxId)
        }
    }
}

impl SigmaSerializable for ErgoBox {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        let ergo_tree_bytes = self.ergo_tree.sigma_serialize_bytes();
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
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let box_candidate = ErgoBoxCandidate::parse_body_with_indexed_digests(None, r)?;
        let tx_id = TxId::sigma_parse(r)?;
        let index = r.get_u16()?;
        Ok(ErgoBox::from_box_candidate(&box_candidate, tx_id, index))
    }
}

/// Contains the same fields as `ErgoBox`, except if transaction id and index,
/// that will be calculated after full transaction formation.
/// Use [`ErgoBoxCandidateBuilder`] to create an instance.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ErgoBoxCandidate {
    /// amount of money associated with the box
    pub value: BoxValue,
    /// guarding script, which should be evaluated to true in order to open this box
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    pub tokens: Vec<Token>,
    ///  additional registers the box can carry over
    pub additional_registers: NonMandatoryRegisters,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    pub creation_height: u32,
}

impl ErgoBoxCandidate {
    /// Box serialization with token ids optionally saved in transaction
    /// (in this case only token index is saved)
    pub fn serialize_body_with_indexed_digests<W: SigmaByteWrite>(
        &self,
        token_ids_in_tx: Option<&IndexSet<TokenId>>,
        w: &mut W,
    ) -> Result<(), io::Error> {
        serialize_box_with_indexed_digests(
            &self.value,
            self.ergo_tree.sigma_serialize_bytes(),
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
    ) -> Result<ErgoBoxCandidate, SerializationError> {
        parse_box_with_indexed_digests(digests_in_tx, r)
    }
}

impl SigmaSerializable for ErgoBoxCandidate {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.serialize_body_with_indexed_digests(None, w)
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
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

#[cfg(test)]
mod tests {

    use super::box_value::tests::ArbBoxValueRange;
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;
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
                    Self::from_box_candidate(&box_candidate, tx_id, index)
                })
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    impl Arbitrary for ErgoBoxAssetsData {
        type Parameters = ArbBoxValueRange;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (any_with::<BoxValue>(args), vec(any::<Token>(), 0..3))
                .prop_map(|(value, tokens)| Self { value, tokens })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }

    #[test]
    fn test_sum_tokens_repeating_token_id() {
        let token = force_any_val::<Token>();
        let b = ErgoBoxAssetsData {
            value: BoxValue::SAFE_USER_MIN,
            tokens: vec![token.clone(), token.clone()],
        };
        assert_eq!(
            u64::from(
                *sum_tokens_from_boxes(vec![b.clone(), b].as_slice())
                    .get(&token.token_id)
                    .unwrap()
            ),
            u64::from(token.amount) * 4
        );
    }

    proptest! {

        #[test]
        fn sum_tokens_eq(b in any::<ErgoBoxAssetsData>()) {
            prop_assert_eq!(sum_tokens(b.tokens().as_slice()), sum_tokens_from_boxes(vec![b].as_slice()))
        }

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
