//! THRESHOLD conjunction for sigma proposition

use core::convert::TryInto;

use alloc::string::ToString;
use alloc::vec::Vec;

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
        match curr_k as usize {
            1 => Cor::normalized(sigmas),
            ch if ch == children_left => Cand::normalized(sigmas),
            _ => SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(Cthreshold {
                k: curr_k,
                children: sigmas,
            })),
        }
    }
}

impl core::fmt::Display for Cthreshold {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("atLeast(")?;
        f.write_str(self.k.to_string().as_str())?;
        f.write_str(", (")?;
        for (i, item) in self.children.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            item.fmt(f)?;
        }
        f.write_str(")")
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
        // The wire format carries k as u16 (as in sigmastate, which keeps the
        // full range), but this type's k is a byte: our polynomial arithmetic
        // can take only byte inputs, and Atleast evaluation already rejects a
        // bound above 255. Reject an out-of-range k here instead of silently
        // truncating it into a different threshold (256 as u8 == 0).
        let k = r.get_u16()?;
        let k = u8::try_from(k).map_err(|_| {
            SigmaParsingError::ValueOutOfBounds(format!("Cthreshold k={} does not fit into u8", k))
        })?;
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

#[cfg(test)]
#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::ergo_tree::ErgoTreeVersion;
    use crate::serialization::sigma_byte_writer::SigmaByteWriter;
    use sigma_ser::vlq_encode::WriteSigmaVlqExt;

    /// Hand-build the wire bytes of a Cthreshold proposition with the given
    /// raw (u16) threshold and `children` TrivialProp(true) children.
    fn cthreshold_bytes(k: u16, children: u16) -> Vec<u8> {
        let mut data = Vec::new();
        let mut w = SigmaByteWriter::new(&mut data, None);
        w.with_tree_version(ErgoTreeVersion::MAX_SCRIPT_VERSION, |w| {
            Cthreshold::OP_CODE.sigma_serialize(w)?;
            w.put_u16(k)?;
            w.put_u16(children)?;
            for _ in 0..children {
                OpCode::TRIVIAL_PROP_TRUE.sigma_serialize(w)?;
            }
            Ok::<(), SigmaParsingError>(())
        })
        .unwrap();
        data
    }

    fn parse_cthreshold(k: u16, children: u16) -> Result<Cthreshold, SigmaParsingError> {
        match SigmaBoolean::sigma_parse_bytes(&cthreshold_bytes(k, children))? {
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(c)) => Ok(c),
            other => panic!("expected Cthreshold, got {:?}", other),
        }
    }

    #[test]
    fn parse_k_above_byte_range_is_rejected() {
        // The reference implementation keeps k in the full u16 range, while
        // this implementation's domain is a byte (see `reduce`); a wire k that
        // does not fit must be rejected instead of silently truncated
        // (256 as u8 == 0, 300 as u8 == 44).
        for k in [256u16, 300, u16::MAX] {
            let res = parse_cthreshold(k, 2);
            assert!(
                matches!(res, Err(SigmaParsingError::ValueOutOfBounds(_))),
                "k={} must be rejected as out of bounds, got {:?}",
                k,
                res
            );
        }
    }

    #[test]
    fn parse_k_within_byte_range_is_preserved() {
        for k in [0u16, 1, 200, 255] {
            let c = parse_cthreshold(k, 2)
                .unwrap_or_else(|e| panic!("k={} must parse, got {:?}", k, e));
            assert_eq!(c.k as u16, k);
            assert_eq!(c.children.len(), 2);
        }
    }
}
