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
        // sigmastate reads `k` and `n` with `getUShort`, which caps neither, parses every
        // child, and only then builds the node: its constructor's
        // `require(k >= 0 && k <= children.length && children.length <= 255)` is the only
        // bound (v6.0.6 `SigmaBoolean.scala:94-100`, `:221-223`). Narrowing `k` here would
        // reject bytes the reference reads further into, so `k` stays wide until the
        // children are read.
        let k = r.get_u16()?;
        let items_count = r.get_u16()?;
        let mut items = Vec::new();
        for _ in 0..items_count {
            items.push(SigmaBoolean::sigma_parse(r)?);
        }
        let children: SigmaConjectureItems<SigmaBoolean> = items.try_into()?;
        // `SigmaConjectureItems` already holds `children.len() <= 255`, so `k <= n` is the
        // whole of the reference's requirement: it rules out a `k` above 255 as well, and it
        // keeps the proof parser's `n_children - k` from underflowing downstream (which
        // would abort the process on a huge allocation).
        if usize::from(k) > children.len() {
            return Err(SigmaParsingError::ValueOutOfBounds(format!(
                "Cthreshold threshold k={0} exceeds children count n={1}",
                k,
                children.len()
            )));
        }
        Ok(Cthreshold {
            // `k <= children.len() <= 255` holds by the check above
            k: k as u8,
            children,
        })
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
        let children: Vec<SigmaBoolean> = (0..n)
            .map(|_| SigmaBoolean::from(ProveDlog::new(g)))
            .collect();
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
        assert_eq!(
            SigmaBoolean::sigma_parse_bytes(&control_bytes).unwrap(),
            control
        );
    }

    // sigmastate reads `k` and `n` with `getUShort`, which caps neither, parses every
    // child, and only then builds the node, whose constructor carries the only bound
    // (v6.0.6 `SigmaBoolean.scala:94-100`, `:221-223`). So a `k` above 255 must not
    // short-circuit the children: the first failure the reference meets is the child's.
    #[test]
    fn a_k_above_255_does_not_pre_empt_the_children() {
        let mut bytes = threshold(1, 1).sigma_serialize_bytes().unwrap();
        // layout: [op_code][k VLQ][n VLQ][children]; k=1 encodes to a single 0x01 byte
        assert_eq!(bytes[1], 0x01);
        // k = 257, VLQ-encoded
        bytes[1] = 0x81;
        bytes.insert(2, 0x02);
        // drop the one declared child, so reading it is the first thing that can fail
        bytes.truncate(4);
        assert!(
            matches!(
                SigmaBoolean::sigma_parse_bytes(&bytes),
                Err(SigmaParsingError::Io(_))
            ),
            "expected the absent child to fail first, got {:?}",
            SigmaBoolean::sigma_parse_bytes(&bytes)
        );
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
