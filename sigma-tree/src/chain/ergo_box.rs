//! Ergo box

mod box_value;
mod register;

use super::{
    token::{TokenAmount, TokenId},
    BoxId, TxId,
};
use crate::{ast::Constant, ergo_tree::ErgoTree};
use indexmap::IndexSet;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::convert::TryFrom;
use std::io;

pub use box_value::BoxValue;

use register::NonMandatoryRegisters;

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
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBox {
    box_id: BoxId,
    /// amount of money associated with the box
    pub value: BoxValue,
    /// guarding script, which should be evaluated to true in order to open this box
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    pub tokens: Vec<TokenAmount>,
    ///  additional registers the box can carry over
    pub additional_registers: NonMandatoryRegisters,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    pub creation_height: u32,
    /// id of transaction which created the box
    pub transaction_id: TxId,
    /// number of box (from 0 to total number of boxes the transaction with transactionId created - 1)
    pub index: u16,
}

impl ErgoBox {
    /// Crate new box
    pub fn new(
        value: BoxValue,
        ergo_tree: ErgoTree,
        tokens: Vec<TokenAmount>,
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
        self.box_id
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
            value: box_candidate.value.clone(),
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
        let mut data = Vec::new();
        self.sigma_serialize(&mut data)
            .expect("ErgoBox serialization failed");
        // TODO: use blake2b256 hash
        BoxId::zero()
    }
}

impl From<&ErgoBox> for ErgoBoxCandidate {
    fn from(b: &ErgoBox) -> Self {
        ErgoBoxCandidate {
            value: b.value.clone(),
            ergo_tree: b.ergo_tree.clone(),
            tokens: b.tokens.clone(),
            additional_registers: b.additional_registers.clone(),
            creation_height: b.creation_height,
        }
    }
}

impl SigmaSerializable for ErgoBox {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        let box_candidate = ErgoBoxCandidate::from(self);
        box_candidate.serialize_body_with_indexed_digests(None, w)?;
        self.transaction_id.sigma_serialize(w)?;
        w.put_u16(self.index)?;
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let box_candidate = ErgoBoxCandidate::parse_body_with_indexed_digests(None, r)?;
        let tx_id = TxId::sigma_parse(r)?;
        let index = r.get_u16()?;
        Ok(ErgoBox::from_box_candidate(&box_candidate, tx_id, index))
    }
}

/// Contains the same fields as `ErgoBox`, except if transaction id and index,
/// that will be calculated after full transaction formation.
#[derive(PartialEq, Eq, Debug)]
pub struct ErgoBoxCandidate {
    /// amount of money associated with the box
    pub value: BoxValue,
    /// guarding script, which should be evaluated to true in order to open this box
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    pub tokens: Vec<TokenAmount>,
    ///  additional registers the box can carry over
    pub additional_registers: NonMandatoryRegisters,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    pub creation_height: u32,
}

impl ErgoBoxCandidate {
    /// create box with value guarded by ErgoTree
    pub fn new(value: BoxValue, ergo_tree: ErgoTree, creation_height: u32) -> ErgoBoxCandidate {
        ErgoBoxCandidate {
            value,
            ergo_tree,
            tokens: vec![],
            additional_registers: NonMandatoryRegisters::empty(),
            creation_height,
        }
    }

    /// Box serialization with token ids optionally saved in transaction
    /// (in this case only token index is saved)
    pub fn serialize_body_with_indexed_digests<W: vlq_encode::WriteSigmaVlqExt>(
        &self,
        token_ids_in_tx: Option<&IndexSet<TokenId>>,
        w: &mut W,
    ) -> Result<(), io::Error> {
        // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoBoxCandidate.scala#L95-L95
        self.value.sigma_serialize(w)?;
        self.ergo_tree.sigma_serialize(w)?;
        w.put_u32(self.creation_height)?;
        w.put_u8(u8::try_from(self.tokens.len()).unwrap())?;

        self.tokens.iter().try_for_each(|t| {
            match token_ids_in_tx {
                Some(token_ids) => w.put_u32(
                    u32::try_from(
                        token_ids
                            .get_full(&t.token_id)
                            // this is not a true runtime error it just means that
                            // calling site messed up the token ids
                            .expect("failed to find token id in tx's digest index")
                            .0,
                    )
                    .unwrap(),
                ),
                None => t.token_id.sigma_serialize(w),
            }
            .and_then(|()| w.put_u64(t.amount))
        })?;

        let regs_num = self.additional_registers.len();
        w.put_u8(regs_num as u8)?;

        self.additional_registers
            .get_ordered_values()
            .into_iter()
            .try_for_each(|c| c.sigma_serialize(w))?;

        Ok(())
    }

    /// Box deserialization with token ids optionally parsed in transaction
    pub fn parse_body_with_indexed_digests<R: vlq_encode::ReadSigmaVlqExt>(
        digests_in_tx: Option<&IndexSet<TokenId>>,
        r: &mut R,
    ) -> Result<ErgoBoxCandidate, SerializationError> {
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
                        Some(i) => Ok(*i),
                        None => Err(SerializationError::Misc(
                            "failed to find token id in tx digests".to_string(),
                        )),
                    }?
                }
            };
            let amount = r.get_u64()?;
            tokens.push(TokenAmount { token_id, amount })
        }

        let regs_num = r.get_u8()?;
        let mut additional_regs = Vec::with_capacity(regs_num as usize);
        for _ in 0..regs_num {
            let v = Constant::sigma_parse(r)?;
            additional_regs.push(v);
        }
        let additional_registers = NonMandatoryRegisters::from_ordered_values(additional_regs)?;
        Ok(ErgoBoxCandidate {
            value,
            ergo_tree,
            tokens,
            additional_registers,
            creation_height,
        })
    }
}

impl SigmaSerializable for ErgoBoxCandidate {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.serialize_body_with_indexed_digests(None, w)
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        ErgoBoxCandidate::parse_body_with_indexed_digests(None, r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};
    use sigma_ser::test_helpers::*;

    impl Arbitrary for ErgoBoxCandidate {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<BoxValue>(),
                any::<ErgoTree>(),
                vec(any::<TokenAmount>(), 0..10),
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
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<ErgoBoxCandidate>(), any::<TxId>(), any::<u16>())
                .prop_map(|(box_candidate, tx_id, index)| {
                    Self::from_box_candidate(&box_candidate, tx_id, index)
                })
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
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
