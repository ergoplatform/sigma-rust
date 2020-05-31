//! Ergo box
use super::token::{TokenAmount, TokenId};
use crate::ergo_tree::ErgoTree;
use indexmap::IndexSet;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;

#[allow(dead_code)]
const STARTING_NON_MANDATORY_INDEX: u8 = 4;

#[derive(PartialEq, Eq, Hash, Debug)]
/// newtype for additional registers R4 - R9
pub struct NonMandatoryRegisterId(u8);

/// Transaction id (ModifierId in sigmastate)
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct TxId(String);

#[derive(PartialEq, Debug)]
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
pub struct ErgoBox {
    /// amount of money associated with the box
    pub value: u64,
    /// guarding script, which should be evaluated to true in order to open this box
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    pub tokens: Vec<TokenAmount>,
    ///  additional registers the box can carry over
    pub additional_registers: HashMap<NonMandatoryRegisterId, Box<[u8]>>,
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
#[derive(PartialEq, Debug)]
pub struct ErgoBoxCandidate {
    /// amount of money associated with the box
    pub value: u64,
    /// guarding script, which should be evaluated to true in order to open this box
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    pub tokens: Vec<TokenAmount>,
    ///  additional registers the box can carry over
    pub additional_registers: HashMap<NonMandatoryRegisterId, Box<[u8]>>,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    pub creation_height: u32,
}

impl ErgoBoxCandidate {
    /// Box serialization with token ids optionally saved in transaction (in this case only token index is saved)
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
                            .expect("failed to find token id in tx's digest index")
                            .0,
                    )
                    .unwrap(),
                ),
                None => t.token_id.sigma_serialize(w),
            }
            .and_then(|()| w.put_u64(t.amount))
        })?;

        assert!(
            self.additional_registers.is_empty(),
            "register serialization is not yet implemented"
        );
        /*
            let regs_num = self.additional_registers.keys().len();
            assert!(
                (regs_num + STARTING_NON_MANDATORY_INDEX as usize) <= 255,
                "The number of non-mandatory indexes exceeds 251 limit."
            );
            w.put_u8(regs_num as u8)?;
        */

        /*
          val nRegs = obj.additionalRegisters.keys.size
          if (nRegs + ErgoBox.startingNonMandatoryIndex > 255)
            sys.error(s"The number of non-mandatory indexes $nRegs exceeds ${255 - ErgoBox.startingNonMandatoryIndex} limit.")
          w.putUByte(nRegs)
          // we assume non-mandatory indexes are densely packed from startingNonMandatoryIndex
          // this convention allows to save 1 bite for each register
          val startReg = ErgoBox.startingNonMandatoryIndex
          val endReg = ErgoBox.startingNonMandatoryIndex + nRegs - 1
          cfor(startReg: Int)(_ <= endReg, _ + 1) { regId =>
            val reg = ErgoBox.findRegisterByIndex(regId.toByte).get
            obj.get(reg) match {
              case Some(v) =>
                w.putValue(v)
              case None =>
                sys.error(s"Set of non-mandatory indexes is not densely packed: " +
                  s"register R$regId is missing in the range [$startReg .. $endReg]")
            }
          }
        */
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

        let additional_registers = HashMap::new();

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
                any::<u64>(),
                any::<ErgoTree>(),
                vec(any::<TokenAmount>(), 0..10),
                any::<u32>(),
            )
                .prop_map(|(value, ergo_tree, tokens, creation_height)| Self {
                    value,
                    ergo_tree,
                    tokens,
                    additional_registers: HashMap::new(),
                    creation_height,
                })
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
