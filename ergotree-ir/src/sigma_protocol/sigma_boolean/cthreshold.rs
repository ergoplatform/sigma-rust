//! THRESHOLD conjunction for sigma proposition

use std::convert::TryInto;

use super::cand::Cand;
use super::cor::Cor;
use super::SigmaBoolean;
use super::SigmaConjecture;
use super::SigmaConjectureItems;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{SigmaParsingError, SigmaSerializable};

// use crate::sigma_protocol::sigma_boolean::SigmaConjecture;

/// THRESHOLD conjunction for sigma proposition
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Cthreshold {
    /// Number of conjectures to be proven
    // Our polynomial arithmetic can take only byte inputs
    pub k: u8,
    /// Items of the proposal
    pub children: SigmaConjectureItems<SigmaBoolean>,
}

impl Cthreshold {
    /// Reduce all possible TrivialProps in the tree
    pub fn reduce(k: u8, children: SigmaConjectureItems<SigmaBoolean>) -> SigmaBoolean {
        if k == 0 {
            return true.into();
        }
        if k as usize > children.len() {
            return false.into();
        }

        let mut curr_k = k;
        let mut children_left = children.len();
        let mut res: Vec<SigmaBoolean> = Vec::new();

        for (i, ch) in children.iter().enumerate() {
            if curr_k == 1 {
                res.append(&mut children.as_vec()[i..children.len()].to_vec());
                // should be 2 or more so unwrap is safe here
                #[allow(clippy::unwrap_used)]
                return Cor::normalized(res.try_into().unwrap());
            }
            if curr_k as usize == children_left {
                res.append(&mut children.as_vec()[i..children.len()].to_vec());
                // should be 2 or more so unwrap is safe here
                #[allow(clippy::unwrap_used)]
                return Cand::normalized(res.try_into().unwrap());
            }
            match ch {
                &SigmaBoolean::TrivialProp(true) => {
                    children_left -= 1;
                    curr_k -= 1;
                }
                &SigmaBoolean::TrivialProp(false) => {
                    children_left -= 1;
                }
                sigma => {
                    res.push(sigma.clone());
                }
            }
        }

        // should be 2 or more so unwrap is safe here
        #[allow(clippy::unwrap_used)]
        let sigmas: SigmaConjectureItems<SigmaBoolean> = res.try_into().unwrap();
        if curr_k == 1 {
            Cor::normalized(sigmas)
        } else if curr_k as usize == children_left {
            Cand::normalized(sigmas)
        } else {
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(Cthreshold {
                k: curr_k,
                children: sigmas,
            }))
        }
    }
}

impl HasStaticOpCode for Cthreshold {
    const OP_CODE: OpCode = OpCode::ATLEAST;
}

impl SigmaSerializable for Cthreshold {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        // put_u16 is used in sigmastate
        // https://github.com/ScorexFoundation/sigmastate-interpreter/blob/e64ca7930ff818403bb3020eadd4b5d8c029d9b6/sigmastate/src/main/scala/sigmastate/Values.scala#L799-L799
        w.put_u16(self.k as u16)?;
        w.put_u16(self.children.len() as u16)?;
        self.children.iter().try_for_each(|i| i.sigma_serialize(w))
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let k = r.get_u16()? as u8; // safe because we serialized u8 as u16
        let items_count = r.get_u16()?;
        let mut items = Vec::with_capacity(items_count as usize);
        for _ in 0..items_count {
            items.push(SigmaBoolean::sigma_parse(r)?);
        }
        Ok(Cthreshold {
            k,
            children: items.try_into()?,
        })
    }
}
