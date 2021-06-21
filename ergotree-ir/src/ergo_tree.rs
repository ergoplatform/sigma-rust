//! ErgoTree
use crate::mir::constant::Constant;
use crate::mir::constant::TryExtractFromError;
use crate::mir::expr::Expr;
use crate::serialization::{
    sigma_byte_reader::{SigmaByteRead, SigmaByteReader},
    sigma_byte_writer::{SigmaByteWrite, SigmaByteWriter},
    SerializationError, SigmaSerializable,
};
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::types::stype::SType;
use io::Cursor;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;

use crate::serialization::constant_store::ConstantStore;
use derive_more::From;
use derive_more::Into;
use std::convert::TryFrom;
use std::io;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;
use thiserror::Error;

#[derive(PartialEq, Eq, Debug, Clone)]
struct ParsedTree {
    constants: Vec<Constant>,
    root: Result<Rc<Expr>, ErgoTreeRootParsingError>,
}

impl ParsedTree {
    /// Sets new constant value for a given index in constants list (as stored in serialized ErgoTree),
    /// and returns either previous constant or None if index is out of bounds
    fn set_constant(&mut self, index: usize, constant: Constant) -> Option<Constant> {
        if index >= self.constants.len() {
            None
        } else {
            let replaced = std::mem::replace(&mut self.constants[index], constant);
            Some(replaced)
        }
    }

    fn template_bytes(&self) -> Vec<u8> {
        match self.root.clone().map(|root| root.sigma_serialize_bytes()) {
            Ok(bytes) => bytes,
            Err(e) => e.root_expr_bytes, // if tree was failed to parse we already have it's bytes
        }
    }

    #[allow(clippy::unwrap_used)] // writer can fail only from OOM, so unwrap is pretty safe here
    fn sigma_serialize_without_size(&self, header: &ErgoTreeHeader) -> Vec<u8> {
        let mut data = Vec::new();
        let mut w = SigmaByteWriter::new(&mut data, None);
        header.sigma_serialize(&mut w).unwrap();
        if header.is_constant_segregation() {
            w.put_usize_as_u32(self.constants.len()).unwrap();
            self.constants
                .iter()
                .try_for_each(|c| c.sigma_serialize(&mut w))
                .unwrap();
        };
        match self.clone().root {
            Ok(expr) => expr.sigma_serialize(&mut w).unwrap(),
            Err(ErgoTreeRootParsingError {
                root_expr_bytes: bytes,
                ..
            }) => w.write_all(&bytes).unwrap(),
        }
        data
    }
}

/// Currently we define meaning for only first byte, which may be extended in future versions.
///  7  6  5  4  3  2  1  0
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
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ErgoTreeHeader(u8);

/// ErgoTree version 0..=7, should fit in 3 bits
#[derive(PartialEq, Eq, Debug, Clone, Into)]
pub struct ErgoTreeVersion(u8);

impl ErgoTreeVersion {
    /// Header mask to extract version bits.
    pub const VERSION_MASK: u8 = 0x07;
    /// Version 0
    pub const V0: Self = ErgoTreeVersion(0);
    /// Version 1 (size flag is mandatory)
    pub const V1: Self = ErgoTreeVersion(1);

    /// Returns a value of the version bits from the given header byte.
    pub fn parse_version(header: &ErgoTreeHeader) -> ErgoTreeVersion {
        let header_byte: u8 = header.clone().into();
        ErgoTreeVersion(header_byte & ErgoTreeVersion::VERSION_MASK)
    }
}

impl ErgoTreeHeader {
    const CONSTANT_SEGREGATION_FLAG: u8 = 0x10;
    const HAS_SIZE_FLAG: u8 = 0x08;

    /// Return a header with version set to 0 and constant segregation flag set to the given value
    pub fn v0(constant_segregation: bool) -> Self {
        if constant_segregation {
            Self::CONSTANT_SEGREGATION_FLAG.into()
        } else {
            ErgoTreeHeader::default()
        }
    }

    /// Return a header with version set to 1 (with size flag set) and constant segregation flag set to the given value
    pub fn v1(constant_segregation: bool) -> Self {
        let version: u8 = ErgoTreeVersion::V1.into();
        // size flag should be set for version > 0
        let mut header_byte: u8 = version | Self::HAS_SIZE_FLAG;
        header_byte = if constant_segregation {
            header_byte | Self::CONSTANT_SEGREGATION_FLAG
        } else {
            header_byte
        };
        header_byte.into()
    }

    /// Returns true if constant segregation flag is set
    pub fn is_constant_segregation(&self) -> bool {
        self.0 & ErgoTreeHeader::CONSTANT_SEGREGATION_FLAG != 0
    }

    /// Returns true if size flag is set
    pub fn has_size(&self) -> bool {
        self.0 & ErgoTreeHeader::HAS_SIZE_FLAG != 0
    }

    /// Returns ErgoTree version
    pub fn version(&self) -> ErgoTreeVersion {
        ErgoTreeVersion::parse_version(self)
    }
}

impl Default for ErgoTreeHeader {
    fn default() -> Self {
        ErgoTreeHeader(ErgoTreeVersion::V0.into())
    }
}

/// Whole ErgoTree parsing (deserialization) error
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoTreeConstantsParsingError {
    /// Ergo tree bytes (failed to deserialize)
    pub bytes: Vec<u8>,
    /// Deserialization error
    pub error: SerializationError,
}

/// ErgoTree root expr parsing (deserialization) error
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoTreeRootParsingError {
    /// Ergo tree root expr bytes (failed to deserialize)
    pub root_expr_bytes: Vec<u8>,
    /// Deserialization error
    pub error: SerializationError,
}

/// ErgoTree parsing (deserialization) error
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ErgoTreeParsingError {
    /// Whole ErgoTree parsing (deserialization) error
    #[error("Whole ErgoTree parsing (deserialization) error")]
    TreeParsingError(ErgoTreeConstantsParsingError),
    /// ErgoTree root expr parsing (deserialization) error
    #[error("ErgoTree root expr parsing (deserialization) error")]
    RootParsingError(ErgoTreeRootParsingError),
}

/** The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoTree {
    header: ErgoTreeHeader,
    tree: Result<ParsedTree, ErgoTreeConstantsParsingError>,
}

impl ErgoTree {
    fn sigma_parse_sized<R: SigmaByteRead>(
        r: &mut R,
        header: ErgoTreeHeader,
        size: u32,
    ) -> Result<Self, SerializationError> {
        let mut buf = vec![0u8; size as usize];
        r.read_exact(buf.as_mut_slice())?;
        if let Ok((constants, mut tree_bytes)) =
            ErgoTree::sigma_parse_tree_bytes(buf.as_mut_slice(), header.is_constant_segregation())
        {
            let tree_bytes_copy = tree_bytes.clone();
            let mut tree_reader = SigmaByteReader::new(
                Cursor::new(&mut tree_bytes[..]),
                ConstantStore::new(constants.clone()),
            );
            match Expr::sigma_parse(&mut tree_reader) {
                Ok(parsed) => Ok(ErgoTree {
                    header,
                    tree: Ok(ParsedTree {
                        constants,
                        root: Ok(Rc::new(parsed)),
                    }),
                }),
                Err(err) => Ok(ErgoTree {
                    header,
                    tree: Ok(ParsedTree {
                        constants,
                        root: Err(ErgoTreeRootParsingError {
                            root_expr_bytes: tree_bytes_copy,
                            error: err,
                        }),
                    }),
                }),
            }
        } else {
            let mut whole_tree_bytes = Vec::new();
            let mut w = SigmaByteWriter::new(&mut whole_tree_bytes, None);
            header.sigma_serialize(&mut w)?;
            if header.has_size() {
                w.put_u32(size)?;
            }
            w.write_all(&buf)?;
            Ok(ErgoTree {
                header,
                tree: Err(ErgoTreeConstantsParsingError {
                    bytes: whole_tree_bytes,
                    error: SerializationError::NotImplementedYet(
                        "not all constant types serialization is supported".to_string(),
                    ),
                }),
            })
        }
    }

    fn sigma_parse_tree_bytes(
        bytes: &mut [u8],
        is_constant_segregation: bool,
    ) -> Result<(Vec<Constant>, Vec<u8>), SerializationError> {
        let mut r = SigmaByteReader::new(Cursor::new(&bytes), ConstantStore::empty());
        let constants = if is_constant_segregation {
            ErgoTree::sigma_parse_constants(&mut r)?
        } else {
            vec![]
        };
        let mut rest_of_the_bytes = Vec::new();
        let _ = r.read_to_end(&mut rest_of_the_bytes);
        Ok((constants, rest_of_the_bytes))
    }

    fn sigma_parse_constants<R: SigmaByteRead>(
        r: &mut R,
    ) -> Result<Vec<Constant>, SerializationError> {
        let constants_len = r.get_u32()?;
        if constants_len as usize > ErgoTree::MAX_CONSTANTS_COUNT {
            return Err(SerializationError::ValueOutOfBounds(
                "too many constants".to_string(),
            ));
        }
        let mut constants = Vec::with_capacity(constants_len as usize);
        for _ in 0..constants_len {
            let c = Constant::sigma_parse(r)?;
            constants.push(c);
        }
        Ok(constants)
    }

    /// Creates a tree using provided header and root expression
    pub fn new(header: ErgoTreeHeader, expr: &Expr) -> Self {
        if header.is_constant_segregation() {
            let mut data = Vec::new();
            let cs = ConstantStore::empty();
            let mut w = SigmaByteWriter::new(&mut data, Some(cs));
            #[allow(clippy::unwrap_used)]
            expr.sigma_serialize(&mut w).unwrap();
            #[allow(clippy::unwrap_used)]
            let constants = w.constant_store_mut_ref().unwrap().get_all();
            let cursor = Cursor::new(&mut data[..]);
            let new_cs = ConstantStore::new(constants.clone());
            let mut sr = SigmaByteReader::new(cursor, new_cs);
            #[allow(clippy::unwrap_used)]
            // if it was serialized, then we should deserialize it without error
            let parsed_expr = Expr::sigma_parse(&mut sr).unwrap();
            ErgoTree {
                header: ErgoTreeHeader(ErgoTreeHeader::CONSTANT_SEGREGATION_FLAG | header.0),
                tree: Ok(ParsedTree {
                    constants,
                    root: Ok(Rc::new(parsed_expr)),
                }),
            }
        } else {
            ErgoTree {
                header,
                tree: Ok(ParsedTree {
                    constants: Vec::new(),
                    root: Ok(Rc::new(expr.clone())),
                }),
            }
        }
    }

    /// Reasonable limit for the number of constants allowed in the ErgoTree
    pub const MAX_CONSTANTS_COUNT: usize = 4096;

    /// get Expr out of ErgoTree
    pub fn proposition(&self) -> Result<Rc<Expr>, ErgoTreeParsingError> {
        let tree = self
            .tree
            .clone()
            .map_err(ErgoTreeParsingError::TreeParsingError)?;
        // This tree has ConstantPlaceholder nodes instead of Constant nodes.
        // We need to substitute placeholders with constant values.
        // So far the easiest way to do it is during deserialization (after the serialization)
        let root = tree.root.map_err(ErgoTreeParsingError::RootParsingError)?;
        if self.header.is_constant_segregation() {
            let mut data = Vec::new();
            let cs = ConstantStore::empty();
            let mut w = SigmaByteWriter::new(&mut data, Some(cs));
            #[allow(clippy::unwrap_used)]
            root.sigma_serialize(&mut w).unwrap();
            let cursor = Cursor::new(&mut data[..]);
            let mut sr = SigmaByteReader::new_with_substitute_placeholders(
                cursor,
                ConstantStore::new(tree.constants),
            );
            #[allow(clippy::unwrap_used)]
            // if it was serialized, then we should deserialize it without error
            let parsed_expr = Expr::sigma_parse(&mut sr).unwrap();
            Ok(Rc::new(parsed_expr))
        } else {
            Ok(root)
        }
    }

    /// Prints with newlines
    pub fn debug_tree(&self) -> String {
        let tree = format!("{:#?}", self);
        tree
    }

    /// Returns Base16-encoded serialized bytes
    pub fn to_base16_bytes(&self) -> String {
        let bytes = self.sigma_serialize_bytes();
        base16::encode_lower(&bytes)
    }

    /// Returns constants number as stored in serialized ErgoTree or error if the parsing of
    /// constants is failed
    pub fn constants_len(&self) -> Result<usize, ErgoTreeConstantsParsingError> {
        self.tree
            .as_ref()
            .map(|tree| tree.constants.len())
            .map_err(|e| e.clone())
    }

    /// Returns constant with given index (as stored in serialized ErgoTree)
    /// or None if index is out of bounds
    /// or error if constants parsing were failed
    pub fn get_constant(
        &self,
        index: usize,
    ) -> Result<Option<Constant>, ErgoTreeConstantsParsingError> {
        self.tree
            .as_ref()
            .map(|tree| tree.constants.get(index).cloned())
            .map_err(|e| e.clone())
    }

    /// Sets new constant value for a given index in constants list (as stored in serialized ErgoTree),
    /// and returns previous constant or None if index is out of bounds
    /// or error if constants parsing were failed
    pub fn set_constant(
        &mut self,
        index: usize,
        constant: Constant,
    ) -> Result<Option<Constant>, ErgoTreeConstantsParsingError> {
        self.tree
            .as_mut()
            .map(|tree| tree.set_constant(index, constant))
            .map_err(|e| e.clone())
    }

    /// Serialized proposition expression of SigmaProp type with
    /// ConstantPlaceholder nodes instead of Constant nodes
    pub fn template_bytes(&self) -> Result<Vec<u8>, ErgoTreeConstantsParsingError> {
        self.tree.clone().map(|tree| tree.template_bytes())
    }
}

impl From<Expr> for ErgoTree {
    fn from(expr: Expr) -> Self {
        match &expr {
            Expr::Const(c) => match c {
                Constant { tpe, .. } if *tpe == SType::SSigmaProp => {
                    ErgoTree::new(ErgoTreeHeader::v0(false), &expr)
                }
                _ => ErgoTree::new(ErgoTreeHeader::v0(true), &expr),
            },
            _ => ErgoTree::new(ErgoTreeHeader::v0(true), &expr),
        }
    }
}
impl SigmaSerializable for ErgoTreeHeader {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u8(self.0)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let header = r.get_u8()?;
        Ok(ErgoTreeHeader(header))
    }
}

impl SigmaSerializable for ErgoTree {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        match &self.tree {
            Ok(parsed_tree) => {
                let bytes = parsed_tree.sigma_serialize_without_size(&self.header);
                if self.header.has_size() {
                    self.header.sigma_serialize(w)?;
                    w.put_usize_as_u32(bytes.len() - 1)?; // skip the header byte
                    w.write_all(&bytes[1..])?; // skip the header byte
                } else {
                    w.write_all(&bytes)?;
                }
            }
            Err(ErgoTreeConstantsParsingError { bytes, .. }) => w.write_all(&bytes[..])?,
        }
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let header = ErgoTreeHeader::sigma_parse(r)?;
        if header.has_size() {
            let tree_size_bytes = r.get_u32()?;
            ErgoTree::sigma_parse_sized(r, header, tree_size_bytes)
        } else {
            let constants = if header.is_constant_segregation() {
                ErgoTree::sigma_parse_constants(r)?
            } else {
                vec![]
            };
            r.set_constant_store(ConstantStore::new(constants.clone()));
            let root = Expr::sigma_parse(r)?;
            Ok(ErgoTree {
                header,
                tree: Ok(ParsedTree {
                    constants,
                    root: Ok(Rc::new(root)),
                }),
            })
        }
    }

    fn sigma_parse_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        let cursor = Cursor::new(bytes);
        let mut r = SigmaByteReader::new(cursor, ConstantStore::empty());
        let header = ErgoTreeHeader::sigma_parse(&mut r)?;
        let rest_of_the_bytes_len = if header.has_size() {
            r.get_u32()?
        } else {
            bytes.len() as u32 - 1 // skip the header byte
        };
        ErgoTree::sigma_parse_sized(&mut r, header, rest_of_the_bytes_len)
    }
}

impl TryFrom<ErgoTree> for ProveDlog {
    type Error = TryExtractFromError;

    fn try_from(tree: ErgoTree) -> Result<Self, Self::Error> {
        let expr = &*tree
            .proposition()
            .map_err(|_| TryExtractFromError("cannot read root expr".to_string()))?;
        match expr {
            Expr::Const(Constant {
                tpe: SType::SSigmaProp,
                v,
            }) => ProveDlog::try_from(v.clone()),
            _ => Err(TryExtractFromError(
                "expected ProveDlog in the root".to_string(),
            )),
        }
    }
}

#[cfg(feature = "arbitrary")]
pub(crate) mod arbitrary {

    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for ErgoTree {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            // make sure that P2PK tree is included
            prop_oneof![
                any::<ProveDlog>()
                    .prop_map(|p| ErgoTree::new(ErgoTreeHeader::v0(false), &Expr::Const(p.into()))),
                any::<ProveDlog>()
                    .prop_map(|p| ErgoTree::new(ErgoTreeHeader::v1(false), &Expr::Const(p.into()))),
                // SigmaProp with constant segregation using both v0 and v1 versions
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SSigmaProp,
                    depth: 1
                })
                .prop_map(|e| ErgoTree::new(ErgoTreeHeader::v1(true), &e)),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SSigmaProp,
                    depth: 1
                })
                .prop_map(|e| ErgoTree::new(ErgoTreeHeader::v0(true), &e)),
            ]
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::address::AddressEncoder;
    use crate::address::NetworkPrefix;
    use crate::mir::value::Value;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ErgoTree>()) {
            dbg!(&v);
            let mut data = Vec::new();
            let mut w = SigmaByteWriter::new(&mut data, None);
            v.sigma_serialize(&mut w).expect("serialization failed");
            // sigma_parse
            let cursor = Cursor::new(&mut data[..]);
            let mut sr = SigmaByteReader::new(cursor, ConstantStore::empty());
            let res = ErgoTree::sigma_parse(&mut sr).expect("parse failed");
            prop_assert_eq!(&res.template_bytes().unwrap(), &v.template_bytes().unwrap());
            prop_assert_eq![&res, &v];
            // sigma_parse_bytes
            let res = ErgoTree::sigma_parse_bytes(&data).expect("parse failed");
            prop_assert_eq!(&res.template_bytes().unwrap(), &v.template_bytes().unwrap());
            prop_assert_eq![res, v];
        }
    }

    #[test]
    fn deserialization_non_parseable_tree_v0() {
        // constants length is set, invalid constant
        let bytes = [
            ErgoTreeHeader::v0(true).into(),
            1, // constants quantity
            0, // invalid constant type
            99,
            99,
        ];
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        assert!(tree.tree.is_err(), "parsing constants should fail");
        assert_eq!(
            tree.sigma_serialize_bytes(),
            bytes,
            "serialization should return original bytes"
        );
        assert!(
            tree.template_bytes().is_err(),
            "template bytes should not be parsed"
        );
    }

    #[test]
    fn deserialization_non_parseable_tree_v1() {
        // v1(size is set), constants length is set, invalid constant
        let bytes = [
            ErgoTreeHeader::v1(true).into(),
            4, // tree size
            1, // constants quantity
            0, // invalid constant type
            99,
            99,
        ];
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        assert!(tree.tree.is_err(), "parsing constants should fail");
        assert_eq!(
            tree.sigma_serialize_bytes(),
            bytes,
            "serialization should return original bytes"
        );
        assert!(
            tree.template_bytes().is_err(),
            "template bytes should not be parsed"
        );
    }

    #[test]
    fn deserialization_non_parseable_root_v0() {
        // no constant segregation, Expr is invalid
        let bytes = [ErgoTreeHeader::v0(false).into(), 0, 1];
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        assert!(
            tree.tree.clone().unwrap().root.is_err(),
            "parsing root should fail"
        );
        assert_eq!(
            tree.sigma_serialize_bytes(),
            bytes,
            "serialization should return original bytes"
        );
        assert_eq!(
            tree.template_bytes().unwrap(),
            bytes[1..],
            "template bytes should be parsed"
        );
    }

    #[test]
    fn deserialization_non_parseable_root_v1() {
        // no constant segregation, Expr is invalid
        let bytes = [
            ErgoTreeHeader::v1(false).into(),
            2, // tree size
            0,
            1,
        ];
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        assert!(
            tree.tree.clone().unwrap().root.is_err(),
            "parsing root should fail"
        );
        assert_eq!(
            tree.sigma_serialize_bytes(),
            bytes,
            "serialization should return original bytes"
        );
        assert_eq!(
            tree.template_bytes().unwrap(),
            bytes[2..],
            "template bytes should be parsed"
        );
    }

    #[test]
    fn test_constant_segregation_header_flag_support() {
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let address = encoder
            .parse_address_from_str("9hzP24a2q8KLPVCUk7gdMDXYc7vinmGuxmLp5KU7k9UwptgYBYV")
            .unwrap();
        let bytes = address.script().unwrap().sigma_serialize_bytes();
        assert_eq!(&bytes[..2], vec![0u8, 8u8].as_slice());
    }

    #[test]
    fn test_constant_segregation() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Value::Boolean(true),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::default(), &expr);
        let bytes = ergo_tree.sigma_serialize_bytes();
        let parsed_expr = ErgoTree::sigma_parse_bytes(&bytes)
            .unwrap()
            .proposition()
            .unwrap();
        assert_eq!(*parsed_expr, expr)
    }

    #[test]
    fn test_constant_len() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Value::Boolean(false),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr);
        assert_eq!(ergo_tree.constants_len().unwrap(), 1);
    }

    #[test]
    fn test_get_constant() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Value::Boolean(false),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr);
        assert_eq!(ergo_tree.constants_len().unwrap(), 1);
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), false.into());
    }

    #[test]
    fn test_set_constant() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Value::Boolean(false),
        });
        let mut ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr);
        assert_eq!(
            ergo_tree.set_constant(0, true.into()).unwrap().unwrap(),
            false.into()
        );
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), true.into());
    }

    #[test]
    fn dex_t2tpool_parse() {
        let base16_str = "19a3030f0400040204020404040404060406058080a0f6f4acdbe01b058080a0f6f4acdbe01b050004d00f0400040005000500d81ad601b2a5730000d602e4c6a70405d603db63087201d604db6308a7d605b27203730100d606b27204730200d607b27203730300d608b27204730400d609b27203730500d60ab27204730600d60b9973078c720602d60c999973088c720502720bd60d8c720802d60e998c720702720dd60f91720e7309d6108c720a02d6117e721006d6127e720e06d613998c7209027210d6147e720d06d615730ad6167e721306d6177e720c06d6187e720b06d6199c72127218d61a9c72167218d1edededededed93c27201c2a793e4c672010405720292c17201c1a793b27203730b00b27204730c00938c7205018c720601ed938c7207018c720801938c7209018c720a019593720c730d95720f929c9c721172127e7202069c7ef07213069a9c72147e7215067e9c720e720206929c9c721472167e7202069c7ef0720e069a9c72117e7215067e9c721372020695ed720f917213730e907217a19d721972149d721a7211ed9272199c7217721492721a9c72177211";
        let tree_bytes = base16::decode(base16_str.as_bytes()).unwrap();
        let mut tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();
        dbg!(&tree);
        assert!(tree.header.has_size());
        assert!(tree.header.is_constant_segregation());
        assert_eq!(tree.header.version(), ErgoTreeVersion::V1);
        tree.set_constant(7, 1i64.into()).unwrap();
        assert_eq!(tree.get_constant(7).unwrap().unwrap(), 1i64.into());
        tree.set_constant(8, 2i64.into()).unwrap();
        assert_eq!(tree.get_constant(8).unwrap().unwrap(), 2i64.into());
        assert!(tree.sigma_serialize_bytes().len() > 1);
    }
}
