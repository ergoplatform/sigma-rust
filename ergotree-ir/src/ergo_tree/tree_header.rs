//! ErgoTree header

use alloc::string::{String, ToString};
use derive_more::{From, Into};
use thiserror::Error;

use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;

/// Currently we define meaning for only first byte, which may be extended in future versions.
///    7  6  5  4  3  2  1  0
///  -------------------------
///  |  |  |  |  |  |  |  |  |
///  -------------------------
///  Bit 7 == 1 if the header contains more than 1 byte (default == 0)
///  Bit 6 - reserved for GZIP compression (should be 0)
///  Bit 5 == 1 - reserved for context dependent costing (should be = 0)
///  Bit 4 == 1 if constant segregation is used for this ErgoTree (default = 0)
///  (see <https://github.com/ScorexFoundation/sigmastate-interpreter/issues/264>)
///  Bit 3 == 1 if size of the whole tree is serialized after the header byte (default = 0)
///  Bits 2-0 - language version (current version == 0)
///
///  Currently we don't specify interpretation for the second and other bytes of the header.
///  We reserve the possibility to extend header by using Bit 7 == 1 and chain additional bytes as in VLQ.
///  Once the new bytes are required, a new version of the language should be created and implemented.
///  That new language will give an interpretation for the new bytes.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoTreeHeader {
    version: ErgoTreeVersion,
    is_constant_segregation: bool,
    has_size: bool,
    reserved_bits: u8,
}

impl ErgoTreeHeader {
    /// Serialization
    pub fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), core3::io::Error> {
        w.put_u8(self.serialized())
    }
    /// Deserialization
    pub fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, ErgoTreeHeaderError> {
        let header_byte = r
            .get_u8()
            .map_err(|e| ErgoTreeHeaderError::IoError(e.to_string()))?;

        ErgoTreeHeader::new(header_byte)
    }
}

impl ErgoTreeHeader {
    const CONSTANT_SEGREGATION_FLAG: u8 = 0b0001_0000;
    const HAS_SIZE_FLAG: u8 = 0b0000_1000;
    const RESERVED_BITS_MASK: u8 = 0b1110_0000;

    /// Parse from byte
    pub fn new(header_byte: u8) -> Result<Self, ErgoTreeHeaderError> {
        let version = ErgoTreeVersion::parse_version(header_byte);
        let has_size = header_byte & Self::HAS_SIZE_FLAG != 0;
        let is_constant_segregation = header_byte & Self::CONSTANT_SEGREGATION_FLAG != 0;
        let reserved_bits = header_byte & Self::RESERVED_BITS_MASK;
        Ok(ErgoTreeHeader {
            version,
            is_constant_segregation,
            has_size,
            reserved_bits,
        })
    }

    /// Serialize to byte
    pub fn serialized(&self) -> u8 {
        let mut header_byte: u8 = self.version.0 | self.reserved_bits;
        if self.is_constant_segregation {
            header_byte |= Self::CONSTANT_SEGREGATION_FLAG;
        }
        if self.has_size {
            header_byte |= Self::HAS_SIZE_FLAG;
        }
        header_byte
    }

    /// Return a header with version set to 0 and constant segregation flag set to the given value
    pub fn v0(constant_segregation: bool) -> Self {
        ErgoTreeHeader {
            version: ErgoTreeVersion::V0,
            is_constant_segregation: constant_segregation,
            has_size: false,
            reserved_bits: 0,
        }
    }

    /// Return a header with version set to 1 (with size flag set) and constant segregation flag set to the given value
    pub fn v1(constant_segregation: bool) -> Self {
        ErgoTreeHeader {
            version: ErgoTreeVersion::V1,
            is_constant_segregation: constant_segregation,
            has_size: true,
            reserved_bits: 0,
        }
    }

    /// Returns true if constant segregation flag is set
    pub fn is_constant_segregation(&self) -> bool {
        self.is_constant_segregation
    }

    /// Returns true if size flag is set
    pub fn has_size(&self) -> bool {
        self.has_size
    }

    /// Returns ErgoTree version
    pub fn version(&self) -> ErgoTreeVersion {
        self.version
    }
}

/// Header parsing error
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum ErgoTreeHeaderError {
    /// Invalid version
    #[error("Invalid version: {0}")]
    VersionError(ErgoTreeVersionError),
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
}

/// ErgoTree version 0..=7, should fit in 3 bits
#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Clone, Copy, From, Into, Default)]
pub struct ErgoTreeVersion(u8);

impl ErgoTreeVersion {
    /// Header mask to extract version bits.
    pub const VERSION_MASK: u8 = 0x07;

    /// Max version of ErgoTree supported by interpreter
    pub const MAX_SCRIPT_VERSION: Self = Self::V3;
    /// Version 0
    pub const V0: Self = ErgoTreeVersion(0);
    /// Version 1 (size flag is mandatory)
    pub const V1: Self = ErgoTreeVersion(1);
    /// Version 2 (JIT)
    pub const V2: Self = ErgoTreeVersion(2);
    /// Version 3 (v6.0/Evolution)
    pub const V3: Self = ErgoTreeVersion(3);

    /// Returns a value of the version bits from the given header byte.
    pub fn parse_version(header_byte: u8) -> Self {
        ErgoTreeVersion(header_byte & ErgoTreeVersion::VERSION_MASK)
    }
}

/// Version parsing error
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum ErgoTreeVersionError {
    /// Invalid version
    #[error("Invalid version: {0}")]
    InvalidVersion(u8),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ergo_tree::ErgoTree;
    use crate::serialization::sigma_byte_reader;
    use crate::serialization::sigma_byte_writer::SigmaByteWriter;
    use crate::serialization::SigmaSerializable;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn preserves_reserved_high_bits_on_roundtrip() {
        for raw in 0u8..=u8::MAX {
            let h = ErgoTreeHeader::new(raw).unwrap();
            assert_eq!(h.serialized(), raw);
            let mut buf = Vec::new();
            let mut w = SigmaByteWriter::new(&mut buf, None);
            h.sigma_serialize(&mut w).unwrap();
            assert_eq!(buf, vec![raw]);
            let mut r = sigma_byte_reader::from_bytes(&buf);
            assert_eq!(ErgoTreeHeader::sigma_parse(&mut r).unwrap(), h);
            assert_eq!(h.version(), ErgoTreeVersion::parse_version(raw));
            assert_eq!(h.has_size(), raw & 0x08 != 0);
            assert_eq!(h.is_constant_segregation(), raw & 0x10 != 0);
        }
        assert_ne!(
            ErgoTreeHeader::new(0x00).unwrap(),
            ErgoTreeHeader::new(0x20).unwrap()
        );
        // End-to-end ErgoTree parse/serialize roundtrip with reserved high bit (0x20) set
        // 0x20 = header (v0, no size, no constant segregation, reserved bit 5 set)
        // 0x08 = TypeCode::SSIGMAPROP, 0xd3 = OpCode::TRIVIAL_PROP_TRUE
        let raw_tree = [0x20u8, 0x08, 0xd3];
        let mut r = sigma_byte_reader::from_bytes(&raw_tree);
        let tree = ErgoTree::sigma_parse(&mut r).unwrap();
        assert_eq!(tree.header().unwrap().serialized(), 0x20);
        assert_eq!(tree.sigma_serialize_bytes().unwrap(), raw_tree);
    }
}
