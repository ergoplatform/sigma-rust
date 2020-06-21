//! Ergo box
use super::token::{TokenAmount, TokenId};
use crate::{ast::Constant, ergo_tree::ErgoTree};
use indexmap::IndexSet;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;

/// newtype for additional registers R4 - R9
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct NonMandatoryRegisterId(u8);

impl NonMandatoryRegisterId {
    /// starting index for non-mandatory registers
    pub const STARTING_INDEX: u8 = 4;

    /// register R4
    pub const R4: NonMandatoryRegisterId = NonMandatoryRegisterId(4);
    /// register R5
    pub const R5: NonMandatoryRegisterId = NonMandatoryRegisterId(5);
    /// register R6
    pub const R6: NonMandatoryRegisterId = NonMandatoryRegisterId(6);
    /// register R7
    pub const R7: NonMandatoryRegisterId = NonMandatoryRegisterId(7);
    /// register R8
    pub const R8: NonMandatoryRegisterId = NonMandatoryRegisterId(8);
    /// register R9
    pub const R9: NonMandatoryRegisterId = NonMandatoryRegisterId(9);

    const REGS: [NonMandatoryRegisterId; 6] = [
        NonMandatoryRegisterId::R4,
        NonMandatoryRegisterId::R5,
        NonMandatoryRegisterId::R6,
        NonMandatoryRegisterId::R7,
        NonMandatoryRegisterId::R8,
        NonMandatoryRegisterId::R9,
    ];

    /// get register by it's index
    /// `i` is expected to be 4 - 9, otherwise it panics
    pub fn find_by_index(i: u8) -> NonMandatoryRegisterId {
        assert!(i >= 4 && i <= 9);
        NonMandatoryRegisterId::REGS[i as usize - 4].clone()
    }
}

/// Transaction id (ModifierId in sigmastate)
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct TxId(String);

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
/// Register R3 contains height when block got included into the blockchain and also transaction
/// identifier and box index in the transaction outputs.
/// Registers R4-R9 are free for arbitrary usage.
///
/// A transaction is unsealing a box. As a box can not be open twice, any further valid transaction
/// can not be linked to the same box.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoBox {
    /// amount of money associated with the box
    pub value: u64,
    /// guarding script, which should be evaluated to true in order to open this box
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    pub tokens: Vec<TokenAmount>,
    ///  additional registers the box can carry over
    pub additional_registers: HashMap<NonMandatoryRegisterId, Box<Constant>>,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    pub creation_height: u32,
    /// id of transaction which created the box
    pub transaction_id: TxId,
    /// number of box (from 0 to total number of boxes the transaction with transactionId created - 1)
    pub index: u16,
}

/// Contains the same fields as `ErgoBox`, except if transaction id and index,
/// that will be calculated after full transaction formation.
#[derive(PartialEq, Eq, Debug)]
pub struct ErgoBoxCandidate {
    /// amount of money associated with the box
    pub value: u64,
    /// guarding script, which should be evaluated to true in order to open this box
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    pub tokens: Vec<TokenAmount>,
    ///  additional registers the box can carry over
    pub additional_registers: HashMap<NonMandatoryRegisterId, Box<Constant>>,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    pub creation_height: u32,
}

impl ErgoBoxCandidate {
    /// create box with value guarded by ErgoTree
    pub fn new(value: u64, ergo_tree: ErgoTree, creation_height: u32) -> ErgoBoxCandidate {
        ErgoBoxCandidate {
            value,
            ergo_tree,
            tokens: vec![],
            additional_registers: HashMap::new(),
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
        w.put_u64(self.value)?;
        self.ergo_tree.sigma_serialize(w)?;
        w.put_u32(self.creation_height)?;
        w.put_u8(u8::try_from(self.tokens.len()).unwrap())?;

        self.tokens.iter().try_for_each(|t| {
            match token_ids_in_tx {
                Some(token_ids) => w.put_u32(
                    u32::try_from(
                        token_ids
                            .get_full(&t.token_id)
                            .expect("failed to find token id in tx's digest index") // TODO: return custom error
                            .0,
                    )
                    .unwrap(),
                ),
                None => t.token_id.sigma_serialize(w),
            }
            .and_then(|()| w.put_u64(t.amount))
        })?;

        let regs_num = self.additional_registers.keys().len();
        assert!(
            (regs_num + NonMandatoryRegisterId::STARTING_INDEX as usize) <= 255,
            "The number of non-mandatory indexes exceeds 251 limit."
        );
        w.put_u8(regs_num as u8)?;
        // we assume non-mandatory indexes are densely packed from startingNonMandatoryIndex
        // this convention allows to save 1 byte for each register
        let start_reg = NonMandatoryRegisterId::STARTING_INDEX;
        let end_reg = start_reg + regs_num as u8;
        for reg_index in start_reg..end_reg {
            let reg_id = NonMandatoryRegisterId::find_by_index(reg_index);
            match self.additional_registers.get(&reg_id) {
                Some(v) => v.sigma_serialize(w),
                None => {
                    let error_msg = format!("Set of non-mandatory indexes is not densely packed: register {} is missing in the range [{} .. {}]", reg_index, start_reg, end_reg);
                    let custom_error = io::Error::new(io::ErrorKind::Other, error_msg);
                    // TODO: change sig to custom error type (non io::Error)
                    Err(custom_error)
                }
            }?;
        }
        Ok(())
    }

    /// Box deserialization with token ids optionally parsed in transaction
    pub fn parse_body_with_indexed_digests<R: vlq_encode::ReadSigmaVlqExt>(
        digests_in_tx: Option<&IndexSet<TokenId>>,
        r: &mut R,
    ) -> Result<ErgoBoxCandidate, SerializationError> {
        // reference implementation -https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoBoxCandidate.scala#L144-L144

        let value = r.get_u64()?;
        let ergo_tree = ErgoTree::sigma_parse(r)?;
        let creation_height = r.get_u32()?;
        let tokens_count = r.get_u8()?;
        let mut tokens = Vec::with_capacity(tokens_count as usize);
        for _ in 0..tokens_count {
            let token_id = match digests_in_tx {
                None => TokenId::sigma_parse(r)?,
                Some(digests) => {
                    let digest_index = r.get_u32()?;
                    *digests
                        .get_index(digest_index as usize)
                        .expect("failed to find token id in tx digests")
                }
            };
            let amount = r.get_u64()?;
            tokens.push(TokenAmount { token_id, amount })
        }

        let regs_num = r.get_u8()?;

        let start_reg = NonMandatoryRegisterId::STARTING_INDEX;
        let end_reg = start_reg + regs_num;

        let mut additional_registers = HashMap::with_capacity(regs_num as usize);
        for reg_index in start_reg..end_reg {
            let reg_id = NonMandatoryRegisterId::find_by_index(reg_index);
            let v = Constant::sigma_parse(r)?;
            additional_registers.insert(reg_id, Box::new(v));
        }

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

#[cfg(feature = "with-serde")]
impl serde::Serialize for ErgoBoxCandidate {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // not implmented
        s.serialize_str("")
    }
}

#[cfg(feature = "with-serde")]
impl<'de> serde::Deserialize<'de> for ErgoBoxCandidate {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
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
                any::<u64>(),
                any::<ErgoTree>(),
                vec(any::<TokenAmount>(), 0..10),
                any::<u32>(),
                vec(any::<Constant>(), 0..7),
            )
                .prop_map(
                    |(value, ergo_tree, tokens, creation_height, constants)| Self {
                        value,
                        ergo_tree,
                        tokens,
                        additional_registers: constants
                            .into_iter()
                            .enumerate()
                            .map(|c| {
                                (
                                    NonMandatoryRegisterId::find_by_index(c.0 as u8 + 4),
                                    Box::new(c.1),
                                )
                            })
                            .collect(),
                        creation_height,
                    },
                )
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ErgoBoxCandidate>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
