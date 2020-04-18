use crate::ergo_tree::ErgoTree;
use crate::{token_id::TokenId, token_info::TokenInfo};
use indexmap::IndexSet;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;

#[allow(dead_code)]
const STARTING_NON_MANDATORY_INDEX: u8 = 4;

#[derive(PartialEq, Eq, Hash)]
pub struct NonMandatoryRegisterId(u8);

pub struct ErgoBoxCandidate {
    pub value: u64,
    pub ergo_tree: ErgoTree,
    pub tokens: Vec<TokenInfo>,
    pub additional_registers: HashMap<NonMandatoryRegisterId, Box<[u8]>>,
    pub creation_height: u32,
}

impl SigmaSerializable for ErgoBoxCandidate {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        serialize_body_with_indexed_digests(self, None, &mut w)
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: R) -> Result<Self, SerializationError> {
        parse_body_with_indexed_digests(None, r)
    }
}

pub fn serialize_body_with_indexed_digests<W: vlq_encode::WriteSigmaVlqExt>(
    b: &ErgoBoxCandidate,
    token_ids_in_tx: Option<&IndexSet<TokenId>>,
    mut w: W,
) -> Result<(), io::Error> {
    // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoBoxCandidate.scala#L95-L95
    w.put_u64(b.value)?;
    b.ergo_tree.sigma_serialize(&mut w)?;
    w.put_u32(b.creation_height)?;
    w.put_u8(u8::try_from(b.tokens.len()).unwrap())?;

    b.tokens.iter().try_for_each(|t| {
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
            None => t.token_id.sigma_serialize(&mut w),
        }
        .and_then(|()| w.put_u64(t.amount))
    })?;

    assert!(
        b.additional_registers.is_empty(),
        "register serialization is not yet implemented"
    );
    /*
        let regs_num = b.additional_registers.keys().len();
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

pub fn parse_body_with_indexed_digests<R: vlq_encode::ReadSigmaVlqExt>(
    digests_in_tx: Option<&IndexSet<TokenId>>,
    mut r: R,
) -> Result<ErgoBoxCandidate, SerializationError> {
    // reference implementation -https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoBoxCandidate.scala#L144-L144

    let value = r.get_u64()?;
    let ergo_tree = ErgoTree::sigma_parse(&mut r)?;
    let creation_height = r.get_u32()?;
    let tokens_count = r.get_u8()?;
    let mut tokens = Vec::with_capacity(tokens_count as usize);
    for _ in 0..(tokens_count - 1) {
        let token_id = match digests_in_tx {
            None => TokenId::sigma_parse(&mut r)?,
            Some(digests) => {
                let digest_index = r.get_u32()?;
                *digests
                    .get_index(digest_index as usize)
                    .expect("failed to find token id in tx digests")
            }
        };
        let amount = r.get_u64()?;
        tokens.push(TokenInfo { token_id, amount })
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
