use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use crate::{
    ast::Constant,
    chain::{
        box_value::BoxValue, register::NonMandatoryRegisters, ErgoBoxCandidate, TokenAmount,
        TokenId,
    },
    ErgoTree,
};
use indexmap::IndexSet;
use sigma_ser::vlq_encode;

use std::convert::TryFrom;
use std::io;

/// Box serialization with token ids optionally saved in transaction
/// (in this case only token index is saved)
pub fn serialize_box_with_indexed_digests<W: vlq_encode::WriteSigmaVlqExt>(
    box_value: &BoxValue,
    ergo_tree_bytes: Vec<u8>,
    tokens: &[TokenAmount],
    additional_registers: &NonMandatoryRegisters,
    creation_height: u32,
    token_ids_in_tx: Option<&IndexSet<TokenId>>,
    w: &mut W,
) -> Result<(), io::Error> {
    // reference implementation - https://github.com/ScorexFoundation/sigmastate-interpreter/blob/9b20cb110effd1987ff76699d637174a4b2fb441/sigmastate/src/main/scala/org/ergoplatform/ErgoBoxCandidate.scala#L95-L95
    box_value.sigma_serialize(w)?;
    w.write_all(&ergo_tree_bytes[..])?;
    w.put_u32(creation_height)?;
    w.put_u8(u8::try_from(tokens.len()).unwrap())?;

    tokens.iter().try_for_each(|t| {
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

    let regs_num = additional_registers.len();
    w.put_u8(regs_num as u8)?;

    additional_registers
        .get_ordered_values()
        .iter()
        .try_for_each(|c| c.sigma_serialize(w))?;

    Ok(())
}

/// Box deserialization with token ids optionally parsed in transaction
pub fn parse_box_with_indexed_digests<R: SigmaByteRead>(
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
                    Some(i) => Ok((*i).clone()),
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
