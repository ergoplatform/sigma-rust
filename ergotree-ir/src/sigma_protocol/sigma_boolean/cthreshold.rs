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
        // The wire carries k as a u16 (the reference implementation stores it
        // as an Int), but this type holds it as a u8: the prover's polynomial
        // arithmetic only takes byte inputs, and Atleast evaluation likewise
        // rejects a bound above 255. An out-of-byte-range k must therefore be
        // rejected, not truncated with `as u8` (256 would silently become
        // k = 0, 300 would become k = 44), which would change the threshold
        // the parsed proposition actually requires.
        let k_wide = r.get_u16()?;
        let k = u8::try_from(k_wide).map_err(|_| {
            SigmaParsingError::ValueOutOfBounds(format!(
                "Cthreshold k={k_wide} exceeds the maximum supported value of 255"
            ))
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
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::serialization::SigmaSerializable;
    use crate::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaConjecture};

    /// Hand-built wire form of a threshold proposition: Cthreshold opcode
    /// (0x98), VLQ-encoded k, VLQ-encoded child count, then one opcode per
    /// child (0xd3 = TrivialProp(true)). The k bytes are written out
    /// literally so the payload does not depend on the serializer under test.
    fn threshold_bytes(k_vlq: &[u8], n_children: u8) -> Vec<u8> {
        let mut bytes = vec![0x98u8];
        bytes.extend_from_slice(k_vlq);
        bytes.push(n_children);
        bytes.extend((0..n_children).map(|_| 0xd3u8));
        bytes
    }

    fn parse(bytes: &[u8]) -> Result<Cthreshold, SigmaParsingError> {
        match SigmaBoolean::sigma_parse_bytes(bytes)? {
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(ct)) => Ok(ct),
            other => panic!("expected Cthreshold, got {other:?}"),
        }
    }

    #[test]
    fn parse_k_256_is_rejected() {
        // VLQ 256 = [0x80, 0x02]; the reference stores k as an Int (256), but
        // truncating it to a byte would silently yield k = 0.
        assert!(parse(&threshold_bytes(&[0x80, 0x02], 2)).is_err());
    }

    #[test]
    fn parse_k_300_is_rejected() {
        // VLQ 300 = [0xAC, 0x02]; truncation would silently yield k = 44, a
        // different threshold than the bytes specify.
        assert!(parse(&threshold_bytes(&[0xAC, 0x02], 2)).is_err());
    }

    #[test]
    fn parse_k_255_is_accepted() {
        // VLQ 255 = [0xFF, 0x01]; the largest byte-sized threshold.
        let ct = parse(&threshold_bytes(&[0xFF, 0x01], 2)).unwrap();
        assert_eq!(ct.k, 255);
        assert_eq!(ct.children.len(), 2);
    }

    #[test]
    fn parse_k_200_is_accepted() {
        // VLQ 200 = [0xC8, 0x01]; above the signed-byte range but a valid u8:
        // the limit is 255 (byte), not 127.
        let ct = parse(&threshold_bytes(&[0xC8, 0x01], 2)).unwrap();
        assert_eq!(ct.k, 200);
    }

    #[test]
    fn roundtrip_k_255() {
        let children: SigmaConjectureItems<SigmaBoolean> = vec![
            SigmaBoolean::TrivialProp(true),
            SigmaBoolean::TrivialProp(false),
        ]
        .try_into()
        .unwrap();
        let sb = SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(Cthreshold {
            k: 255,
            children,
        }));
        let bytes = sb.sigma_serialize_bytes().unwrap();
        assert_eq!(SigmaBoolean::sigma_parse_bytes(&bytes).unwrap(), sb);
    }
}
