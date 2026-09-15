use core::fmt::Formatter;
use core::hash::Hash;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::mir::expr::InvalidArgumentError;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;

/// Type variable for generic signatures
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct STypeVar {
    /// Type variable name (e.g. "T")
    name_bytes: Vec<u8>,
}

impl core::fmt::Debug for STypeVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.as_string().fmt(f)
    }
}

impl STypeVar {
    /// Creates a type variable from a UTF8 text string (name length is a `u8`, so 0..=255 bytes)
    pub fn new_from_str(name: &'static str) -> Result<Self, InvalidArgumentError> {
        Ok(Self {
            name_bytes: name.to_string().into_bytes(),
        })
    }

    /// Creates a type variable from bytes of a UTF8 text string (name length 0..=255).
    ///
    /// Mirrors the JVM `TypeSerializer` (`new String(bytes, UTF_8)`): a non-UTF8 name is
    /// lossily decoded -- malformed bytes become U+FFFD, matching the JVM's substitution
    /// byte-for-byte (see [`jvm_lossy_utf8`]) -- and stored canonicalized, rather than
    /// rejected. (`Result` is kept for API stability; this no longer errors.)
    pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Self, InvalidArgumentError> {
        Ok(Self {
            name_bytes: jvm_lossy_utf8(&bytes),
        })
    }

    /// Returns text representation (e.g "T", etc.)
    pub fn as_string(&self) -> String {
        #[allow(clippy::unwrap_used)]
        String::from_utf8(self.name_bytes.clone()).unwrap()
    }

    /// "T" type variable
    pub fn t() -> Self {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("T").unwrap()
    }

    /// "IV"(Input Value) type variable
    pub fn iv() -> STypeVar {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("IV").unwrap()
    }
    /// "OV"(Output Value) type variable
    pub fn ov() -> STypeVar {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("OV").unwrap()
    }
}

/// Lossily decodes `bytes` as UTF-8 the way the JVM `new String(bytes, UTF_8)` does,
/// returning the canonical UTF-8 bytes (valid input is returned unchanged).
///
/// Deliberately NOT `String::from_utf8_lossy`: Rust replaces per the Unicode
/// "substitution of maximal subparts" rule, whereas the JVM replaces an entire
/// *attempted* multibyte sequence -- a valid lead byte plus the following continuation
/// bytes, up to the sequence's expected length -- with a single U+FFFD. They diverge on,
/// e.g., `ed a0 80` (a UTF-16 surrogate encoded in UTF-8): the JVM yields one U+FFFD,
/// `from_utf8_lossy` yields three. Matching the JVM keeps the re-serialized name -- and
/// thus the tree bytes and script hash -- identical across implementations.
fn jvm_lossy_utf8(bytes: &[u8]) -> Vec<u8> {
    const REPLACEMENT: [u8; 3] = [0xEF, 0xBF, 0xBD]; // U+FFFD in UTF-8
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b < 0x80 {
            out.push(b); // ASCII
            i += 1;
            continue;
        }
        // Expected total length for a valid lead byte. 0xC0/0xC1 and 0xF5..=0xFF are
        // never valid leads; a stray continuation (0x80..=0xBF) is not a lead either.
        let expected = match b {
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF4 => 4,
            _ => 0,
        };
        if expected == 0 {
            out.extend_from_slice(&REPLACEMENT); // one byte, one replacement
            i += 1;
            continue;
        }
        // Take the lead + following continuation bytes, up to `expected` total.
        let mut j = i + 1;
        while j < bytes.len() && j - i < expected && (0x80..=0xBF).contains(&bytes[j]) {
            j += 1;
        }
        let seq = &bytes[i..j];
        if seq.len() == expected && core::str::from_utf8(seq).is_ok() {
            out.extend_from_slice(seq); // a complete, valid char -- bytes unchanged
        } else {
            out.extend_from_slice(&REPLACEMENT); // one replacement for the attempt
        }
        i = j;
    }
    out
}

impl SigmaSerializable for STypeVar {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.name_bytes.len() as u8)?;
        w.write_all(self.name_bytes.as_slice())?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let name_len = r.get_u8()?;
        let mut bytes = vec![0; name_len as usize];
        r.read_exact(&mut bytes)?;
        Ok(STypeVar::new_from_bytes(bytes)?)
    }
}

/// Type parameter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeParam {
    pub(crate) ident: STypeVar,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // The JVM `TypeSerializer.deserialize` reads the type-var name length via
    // `getUByte()` (0..=255, no bound) then `new String(getBytes(len))`, so an
    // empty name (len 0) and a 255-byte name both deserialize. The previous
    // `BoundedVec<u8, 1, 254>` over-rejected both -- a divergence from sigma-state.
    // See sigma-state core/.../serialization/TypeSerializer.scala:202-206.

    #[test]
    fn parse_name_length_0() {
        // u8 length 0x00, zero name bytes -> STypeVar("")
        let tv = STypeVar::sigma_parse_bytes(&[0x00u8]).unwrap();
        assert_eq!(tv.as_string(), "");
        assert_eq!(tv.sigma_serialize_bytes().unwrap(), vec![0x00u8]);
    }

    #[test]
    fn parse_name_length_255() {
        // u8 length 0xff (255), then 255 'a' bytes -> STypeVar("a" * 255)
        let mut bytes = vec![0xffu8];
        bytes.extend_from_slice(&[b'a'; 255]);
        let tv = STypeVar::sigma_parse_bytes(&bytes).unwrap();
        assert_eq!(tv.as_string(), "a".repeat(255));
        assert_eq!(tv.sigma_serialize_bytes().unwrap(), bytes);
    }

    #[test]
    fn name_jvm_lossy_substitution() {
        // A non-UTF8 name must canonicalize exactly as the JVM `new String(bytes, UTF_8)`
        // does, so the re-serialized name (-> tree bytes -> script hash) matches. Oracle:
        // sigma-state 6.0.3 / Java 17 (SANTA byte-exactness table). `from_utf8_lossy` would
        // differ on the surrogate `ed a0 80` (3 U+FFFD vs the JVM's 1).
        // (input bytes, JVM canonical name bytes)
        let cases: &[(&[u8], &[u8])] = &[
            (&[0xff], &[0xef, 0xbf, 0xbd]),
            (&[0xe2, 0x82], &[0xef, 0xbf, 0xbd]),
            (&[0xc0, 0x80], &[0xef, 0xbf, 0xbd, 0xef, 0xbf, 0xbd]),
            (&[0xed, 0xa0, 0x80], &[0xef, 0xbf, 0xbd]),
            (&[0x61, 0xff, 0x62], &[0x61, 0xef, 0xbf, 0xbd, 0x62]),
        ];
        for (input, expected_name) in cases {
            let tv = STypeVar::new_from_bytes(input.to_vec()).unwrap();
            let mut expected_wire = vec![expected_name.len() as u8];
            expected_wire.extend_from_slice(expected_name);
            assert_eq!(
                tv.sigma_serialize_bytes().unwrap(),
                expected_wire,
                "input {:02x?}",
                input
            );
        }
    }

    #[test]
    fn name_valid_utf8_unchanged() {
        // a valid multi-byte name round-trips byte-for-byte (euro sign U+20AC = e2 82 ac)
        let tv = STypeVar::new_from_bytes(vec![0xe2, 0x82, 0xac]).unwrap();
        assert_eq!(tv.as_string(), "\u{20ac}");
        assert_eq!(
            tv.sigma_serialize_bytes().unwrap(),
            vec![0x03, 0xe2, 0x82, 0xac]
        );
    }
}
