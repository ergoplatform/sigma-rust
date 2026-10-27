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
        // k is serialized as u16 (see sigma_serialize); only 0..=255 is valid on the wire.
        // Mirror the JVM reference `require(k <= children.length && children.length <= 255)`
        // and reject out-of-range k instead of silently truncating it.
        let k: u8 = r
            .get_u16()?
            .try_into()
            .map_err(|_| SigmaParsingError::ValueOutOfBounds("Cthreshold threshold k exceeds 255".to_string()))?;
        let items_count = r.get_u16()?;
        let mut items = Vec::with_capacity(items_count as usize);
        for _ in 0..items_count {
            items.push(SigmaBoolean::sigma_parse(r)?);
        }
        let children: SigmaConjectureItems<SigmaBoolean> = items.try_into()?;
        // `k > n` is malformed (the JVM reference rejects it at construction); reject it
        // here so that the proof parser can never hit the `n_children - k` usize
        // underflow downstream (which would abort the process on a huge allocation).
        if k as usize > children.len() {
            return Err(SigmaParsingError::ValueOutOfBounds(format!(
                "Cthreshold threshold k={0} exceeds children count n={1}",
                k, children.len()
            )));
        }
        Ok(Cthreshold { k, children })
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::SigmaSerializable;
    use crate::sigma_protocol::sigma_boolean::ProveDlog;
    use ergo_chain_types::EcPoint;

    const G_HEX: &str = "0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798";

    fn threshold(k: u8, n: usize) -> SigmaBoolean {
        let g = EcPoint::from_base16_str(G_HEX.to_string()).unwrap();
        let children: Vec<SigmaBoolean> =
            (0..n).map(|_| SigmaBoolean::from(ProveDlog::new(g.clone()))).collect();
        SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(Cthreshold {
            k,
            children: children.try_into().unwrap(),
        }))
    }

    // security regression test for GHSA-xgcg-37qp-p8fx:
    // a wire-constructible Cthreshold with k > n must be rejected at parse time,
    // instead of aborting the verifier with a "capacity overflow" panic
    #[test]
    fn test_parse_rejects_k_gt_n() {
        let bytes = threshold(2, 1).sigma_serialize_bytes().unwrap();
        assert!(matches!(
            SigmaBoolean::sigma_parse_bytes(&bytes),
            Err(SigmaParsingError::ValueOutOfBounds(_))
        ));
        // control: a well-formed threshold still parses (and round-trips)
        let control = threshold(1, 2);
        let control_bytes = control.sigma_serialize_bytes().unwrap();
        assert_eq!(SigmaBoolean::sigma_parse_bytes(&control_bytes).unwrap(), control);
    }

    // a wire-declared k over u8::MAX is malformed; previously it was silently truncated
    // via `as u8` and accepted (e.g. 257 parsed as 1), diverging from the JVM reference
    #[test]
    fn test_parse_rejects_k_over_255() {
        let mut bytes = threshold(1, 1).sigma_serialize_bytes().unwrap();
        // layout: [op_code][k VLQ][n VLQ][children]; k=1 encodes to a single 0x01 byte
        assert_eq!(bytes[1], 0x01);
        // replace k=1 with VLQ-encoded 257 = [0x81, 0x02]
        bytes[1] = 0x81;
        bytes.insert(2, 0x02);
        assert!(SigmaBoolean::sigma_parse_bytes(&bytes).is_err());
    }
}
