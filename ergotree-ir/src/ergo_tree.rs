//! ErgoTree
use crate::mir::constant::Constant;
use crate::mir::constant::TryExtractFromError;
use crate::mir::expr::Expr;
use crate::serialization::SigmaSerializationError;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::{SigmaByteRead, SigmaByteReader},
    sigma_byte_writer::{SigmaByteWrite, SigmaByteWriter},
    SigmaParsingError, SigmaSerializable,
};
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::types::stype::SType;
use io::Cursor;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;

use crate::serialization::constant_store::ConstantStore;
use derive_more::From;
use std::convert::TryFrom;
use std::io;
use std::io::Read;
use thiserror::Error;

mod tree_header;
pub use tree_header::*;

/// Parsed ErgoTree
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ParsedErgoTree {
    header: ErgoTreeHeader,
    constants: Vec<Constant>,
    root: Expr,
}

impl ParsedErgoTree {
    /// Returns new ParsedTree with a new constant value for a given index in constants list
    /// (as stored in serialized ErgoTree), or an error
    fn with_constant(self, index: usize, constant: Constant) -> Result<Self, SetConstantError> {
        let mut new_constants = self.constants.clone();
        if let Some(old_constant) = self.constants.get(index) {
            if constant.tpe == old_constant.tpe {
                let _ = std::mem::replace(&mut new_constants[index], constant);
                Ok(Self {
                    constants: new_constants,
                    ..self
                })
            } else {
                Err(SetConstantError::TypeMismatch(format!(
                    "with_constant: expected constant type to be {:?}, got {:?}",
                    old_constant.tpe, constant.tpe
                )))
            }
        } else {
            Err(SetConstantError::OutOfBounds(format!(
                "with_constant: index({0}) out of bounds (lengh = {1})",
                index,
                self.constants.len()
            )))
        }
    }

    fn template_bytes(&self) -> Result<Vec<u8>, ErgoTreeError> {
        Ok(self.root.sigma_serialize_bytes()?)
    }
}

/// Errors on fail to set a new constant value
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum SetConstantError {
    /// Index is out of bounds
    #[error("Index is out of bounds: {0}")]
    OutOfBounds(String),
    /// Existing constant type differs from the provided new constant type
    #[error("Existing constant type differs from the provided new constant type: {0}")]
    TypeMismatch(String),
}

/// ErgoTree root expr parsing (deserialization) error inner
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum ErgoTreeRootParsingError {
    /// ErgoTree root expr parsing (deserialization) error
    #[error("SigmaParsingError: {0:?}")]
    SigmaParsingError(SigmaParsingError),
    /// Non-consumed bytes after root expr is parsed
    #[error("Non-consumed bytes after root expr is parsed")]
    NonConsumedBytes,
}

/// ErgoTree serialization and parsing (deserialization) error
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum ErgoTreeError {
    /// ErgoTree header error
    #[error("ErgoTree header error: {0:?}")]
    HeaderError(ErgoTreeHeaderError),
    /// ErgoTree constants error
    #[error("ErgoTree constants error: {0:?}")]
    ConstantsError(ErgoTreeConstantError),
    /// ErgoTree root expr parsing (deserialization) error
    #[error("ErgoTree root expr parsing (deserialization) error: {0:?}")]
    RootParsingError(ErgoTreeRootParsingError),
    /// ErgoTree serialization error
    #[error("ErgoTree serialization error: {0}")]
    RootSerializationError(SigmaSerializationError),
    /// Sigma parsing error
    #[error("Sigma parsing error: {0:?}")]
    SigmaParsingError(SigmaParsingError),
    /// IO error
    #[error("IO error: {0:?}")]
    IoError(String),
}

/// The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub enum ErgoTree {
    /// Unparsed tree, with original bytes and error
    Unparsed {
        /// Original tree bytes
        tree_bytes: Vec<u8>,
        /// Parsing error
        error: ErgoTreeError,
    },
    /// Parsed tree
    Parsed(ParsedErgoTree),
}

impl ErgoTree {
    fn parsed_tree(&self) -> Result<&ParsedErgoTree, ErgoTreeError> {
        match self {
            ErgoTree::Unparsed {
                tree_bytes: _,
                error,
            } => Err(error.clone()),
            ErgoTree::Parsed(parsed) => Ok(parsed),
        }
    }

    fn sigma_parse_sized<R: SigmaByteRead>(
        r: &mut R,
        header: ErgoTreeHeader,
    ) -> Result<ParsedErgoTree, ErgoTreeError> {
        let constants = if header.is_constant_segregation() {
            ErgoTree::sigma_parse_constants(r)?
        } else {
            vec![]
        };
        r.set_constant_store(ConstantStore::new(constants.clone()));
        let root = Expr::sigma_parse(r)?;
        Ok(ParsedErgoTree {
            header,
            constants,
            root,
        })
    }

    fn sigma_parse_constants<R: SigmaByteRead>(
        r: &mut R,
    ) -> Result<Vec<Constant>, SigmaParsingError> {
        let constants_len = r.get_u32()?;
        if constants_len as usize > ErgoTree::MAX_CONSTANTS_COUNT {
            return Err(SigmaParsingError::ValueOutOfBounds(
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
    pub fn new(header: ErgoTreeHeader, expr: &Expr) -> Result<Self, ErgoTreeError> {
        Ok(if header.is_constant_segregation() {
            let mut data = Vec::new();
            let cs = ConstantStore::empty();
            let mut w = SigmaByteWriter::new(&mut data, Some(cs));
            expr.sigma_serialize(&mut w)?;
            #[allow(clippy::unwrap_used)]
            // We set constant store earlier
            let constants = w.constant_store_mut_ref().unwrap().get_all();
            let cursor = Cursor::new(&mut data[..]);
            let new_cs = ConstantStore::new(constants.clone());
            let mut sr = SigmaByteReader::new(cursor, new_cs);
            let parsed_expr = Expr::sigma_parse(&mut sr)?;
            ErgoTree::Parsed(ParsedErgoTree {
                header,
                constants,
                root: parsed_expr,
            })
        } else {
            ErgoTree::Parsed(ParsedErgoTree {
                header,
                constants: Vec::new(),
                root: expr.clone(),
            })
        })
    }

    /// Reasonable limit for the number of constants allowed in the ErgoTree
    pub const MAX_CONSTANTS_COUNT: usize = 4096;

    /// get Expr out of ErgoTree
    pub fn proposition(&self) -> Result<Expr, ErgoTreeError> {
        let tree = self.parsed_tree()?.clone();
        // This tree has ConstantPlaceholder nodes instead of Constant nodes.
        // We need to substitute placeholders with constant values.
        // So far the easiest way to do it is during deserialization (after the serialization)
        let root = tree.root;
        if tree.header.is_constant_segregation() {
            let mut data = Vec::new();
            let cs = ConstantStore::empty();
            let mut w = SigmaByteWriter::new(&mut data, Some(cs));
            root.sigma_serialize(&mut w)?;
            let cursor = Cursor::new(&mut data[..]);
            let mut sr = SigmaByteReader::new_with_substitute_placeholders(
                cursor,
                ConstantStore::new(tree.constants),
            );
            let parsed_expr = Expr::sigma_parse(&mut sr)?;
            Ok(parsed_expr)
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
    pub fn to_base16_bytes(&self) -> Result<String, SigmaSerializationError> {
        let bytes = self.sigma_serialize_bytes()?;
        Ok(base16::encode_lower(&bytes))
    }

    /// Returns constants number as stored in serialized ErgoTree or error if the parsing of
    /// constants is failed
    pub fn constants_len(&self) -> Result<usize, ErgoTreeError> {
        self.parsed_tree().map(|tree| tree.constants.len())
    }

    /// Returns constant with given index (as stored in serialized ErgoTree)
    /// or None if index is out of bounds
    /// or error if constants parsing were failed
    pub fn get_constant(&self, index: usize) -> Result<Option<Constant>, ErgoTreeError> {
        self.parsed_tree()
            .map(|tree| tree.constants.get(index).cloned())
    }

    /// Returns all constants (as stored in serialized ErgoTree)
    /// or error if constants parsing were failed
    pub fn get_constants(&self) -> Result<Vec<Constant>, ErgoTreeError> {
        self.parsed_tree().map(|tree| tree.constants.clone())
    }

    /// Returns new ErgoTree with a new constant value for a given index in constants list (as
    /// stored in serialized ErgoTree), or an error. Note that the type of the new constant must
    /// coincide with that of the constant being replaced, or an error is returned too.
    pub fn with_constant(self, index: usize, constant: Constant) -> Result<Self, ErgoTreeError> {
        let parsed_tree = self.parsed_tree()?.clone();
        Ok(Self::Parsed(
            parsed_tree
                .with_constant(index, constant)
                .map_err(ErgoTreeConstantError::from)?,
        ))
    }

    /// Serialized proposition expression of SigmaProp type with
    /// ConstantPlaceholder nodes instead of Constant nodes
    pub fn template_bytes(&self) -> Result<Vec<u8>, ErgoTreeError> {
        self.clone().parsed_tree()?.template_bytes()
    }
}

/// Constants related errors
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum ErgoTreeConstantError {
    /// Fail to parse a constant when deserializing an ErgoTree
    #[error("Fail to parse a constant when deserializing an ErgoTree: {0}")]
    ParsingError(SigmaParsingError),
    /// Fail to set a new constant value
    #[error("Fail to set a new constant value: {0}")]
    SetConstantError(SetConstantError),
}

impl TryFrom<Expr> for ErgoTree {
    type Error = ErgoTreeError;

    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
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

impl SigmaSerializable for ErgoTree {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        match self {
            ErgoTree::Unparsed {
                tree_bytes,
                error: _,
            } => w.write_all(&tree_bytes[..])?,
            ErgoTree::Parsed(parsed_tree) => {
                let bytes = {
                    let mut data = Vec::new();
                    let mut inner_w = SigmaByteWriter::new(&mut data, None);
                    if parsed_tree.header.is_constant_segregation() {
                        inner_w.put_usize_as_u32_unwrapped(parsed_tree.constants.len())?;
                        parsed_tree
                            .constants
                            .iter()
                            .try_for_each(|c| c.sigma_serialize(&mut inner_w))?;
                    };
                    parsed_tree.root.sigma_serialize(&mut inner_w)?;
                    data
                };

                parsed_tree.header.sigma_serialize(w)?;
                if parsed_tree.header.has_size() {
                    w.put_usize_as_u32_unwrapped(bytes.len())?;
                }
                w.write_all(&bytes)?;
            }
        };
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let header = ErgoTreeHeader::sigma_parse(r)?;
        if header.has_size() {
            let tree_size_bytes = r.get_u32()?;
            let mut buf = vec![0u8; tree_size_bytes as usize];
            r.read_exact(buf.as_mut_slice())?;
            let buf_copy = buf.clone();
            let mut inner_r =
                SigmaByteReader::new(Cursor::new(&mut buf[..]), ConstantStore::empty());
            match ErgoTree::sigma_parse_sized(&mut inner_r, header.clone()) {
                Ok(parsed_tree) => Ok(parsed_tree.into()),
                Err(error) => {
                    let mut bytes = vec![header.serialized()];
                    #[allow(clippy::unwrap_used)]
                    bytes.put_u32(tree_size_bytes).unwrap();
                    bytes.extend_from_slice(&buf_copy);
                    Ok(ErgoTree::Unparsed {
                        tree_bytes: bytes,
                        error,
                    })
                }
            }
        } else {
            let constants = if header.is_constant_segregation() {
                ErgoTree::sigma_parse_constants(r)?
            } else {
                vec![]
            };
            r.set_constant_store(ConstantStore::new(constants.clone()));
            let root = Expr::sigma_parse(r)?;
            Ok(ErgoTree::Parsed(ParsedErgoTree {
                header,
                constants,
                root,
            }))
        }
    }

    fn sigma_parse_bytes(bytes: &[u8]) -> Result<Self, SigmaParsingError> {
        let wrap_in_ergotree = |r: Result<ParsedErgoTree, ErgoTreeError>| -> Self {
            match r {
                Ok(parsed_tree) => ErgoTree::Parsed(parsed_tree),
                Err(error) => ErgoTree::Unparsed {
                    tree_bytes: bytes.to_vec(),
                    error,
                },
            }
        };
        let mut r = SigmaByteReader::new(Cursor::new(bytes), ConstantStore::empty());
        let tree: Result<ErgoTree, SigmaParsingError> = match ErgoTreeHeader::sigma_parse(&mut r) {
            Ok(header) => {
                if header.has_size() {
                    let tree_size_bytes = r.get_u32()?;
                    let mut buf = vec![0u8; tree_size_bytes as usize];
                    r.read_exact(buf.as_mut_slice())?;
                    let mut inner_r =
                        SigmaByteReader::new(Cursor::new(&mut buf[..]), ConstantStore::empty());
                    Ok(wrap_in_ergotree(ErgoTree::sigma_parse_sized(
                        &mut inner_r,
                        header,
                    )))
                } else {
                    Ok(wrap_in_ergotree(ErgoTree::sigma_parse_sized(
                        &mut r, header,
                    )))
                }
            }
            Err(e) => Ok(ErgoTree::Unparsed {
                tree_bytes: bytes.to_vec(),
                error: e.into(),
            }),
        };
        let mut buffer = Vec::new();
        if let Ok(0) = r.read_to_end(&mut buffer) {
            tree
        } else {
            Ok(ErgoTree::Unparsed {
                tree_bytes: bytes.to_vec(),
                error: ErgoTreeRootParsingError::NonConsumedBytes.into(),
            })
        }
    }
}

impl TryFrom<ErgoTree> for ProveDlog {
    type Error = TryExtractFromError;

    fn try_from(tree: ErgoTree) -> Result<Self, Self::Error> {
        let expr = tree
            .proposition()
            .map_err(|_| TryExtractFromError("cannot read root expr".to_string()))?;
        match expr {
            Expr::Const(Constant {
                tpe: SType::SSigmaProp,
                v,
            }) => ProveDlog::try_from(v),
            _ => Err(TryExtractFromError(
                "expected ProveDlog in the root".to_string(),
            )),
        }
    }
}

impl From<std::io::Error> for ErgoTreeError {
    fn from(e: std::io::Error) -> Self {
        ErgoTreeError::IoError(e.to_string())
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
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
                any::<ProveDlog>().prop_map(|p| ErgoTree::new(
                    ErgoTreeHeader::v0(false),
                    &Expr::Const(p.into())
                )
                .unwrap()),
                any::<ProveDlog>().prop_map(|p| ErgoTree::new(
                    ErgoTreeHeader::v1(false),
                    &Expr::Const(p.into())
                )
                .unwrap()),
                // SigmaProp with constant segregation using both v0 and v1 versions
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SSigmaProp,
                    depth: 1
                })
                .prop_map(|e| ErgoTree::new(ErgoTreeHeader::v1(true), &e).unwrap()),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SSigmaProp,
                    depth: 1
                })
                .prop_map(|e| ErgoTree::new(ErgoTreeHeader::v0(true), &e).unwrap()),
            ]
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::chain::address::AddressEncoder;
    use crate::chain::address::NetworkPrefix;
    use crate::mir::constant::Literal;
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
            // prop_assert_eq!(&res.template_bytes().unwrap(), &v.template_bytes().unwrap());
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
            ErgoTreeHeader::v0(true).serialized(),
            1, // constants quantity
            0, // invalid constant type
            99,
            99,
        ];
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        assert!(tree.parsed_tree().is_err(), "parsing constants should fail");
        assert_eq!(
            tree.sigma_serialize_bytes().unwrap(),
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
            ErgoTreeHeader::v1(true).serialized(),
            4, // tree size
            1, // constants quantity
            0, // invalid constant type
            99,
            99,
        ];
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        assert!(tree.parsed_tree().is_err(), "parsing constants should fail");
        assert_eq!(
            tree.sigma_serialize_bytes().unwrap(),
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
        let bytes = [ErgoTreeHeader::v0(false).serialized(), 0, 1];
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        assert!(tree.parsed_tree().is_err(), "parsing root should fail");
        assert_eq!(
            tree.sigma_serialize_bytes().unwrap(),
            bytes,
            "serialization should return original bytes"
        );
        assert!(
            tree.template_bytes().is_err(),
            "template bytes should not be parsed"
        );
    }

    #[test]
    fn deserialization_non_parseable_root_v1() {
        // no constant segregation, Expr is invalid
        let bytes = [
            ErgoTreeHeader::v1(false).serialized(),
            2, // tree size
            0,
            1,
        ];
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        assert!(tree.parsed_tree().is_err(), "parsing root should fail");
        assert_eq!(
            tree.sigma_serialize_bytes().unwrap(),
            bytes,
            "serialization should return original bytes"
        );
        assert!(
            tree.template_bytes().is_err(),
            "template bytes should not be parsed"
        );
        // parsing via sigma_parse should fail as well
        let mut reader = SigmaByteReader::new(Cursor::new(&bytes), ConstantStore::empty());
        let tree = ErgoTree::sigma_parse(&mut reader).unwrap();
        assert!(tree.parsed_tree().is_err(), "parsing root should fail");
        assert_eq!(
            tree.sigma_serialize_bytes().unwrap(),
            bytes,
            "serialization should return original bytes"
        );
        assert!(
            tree.template_bytes().is_err(),
            "template bytes should not be parsed"
        );
    }

    #[test]
    fn test_constant_segregation_header_flag_support() {
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let address = encoder
            .parse_address_from_str("9hzP24a2q8KLPVCUk7gdMDXYc7vinmGuxmLp5KU7k9UwptgYBYV")
            .unwrap();
        let bytes = address.script().unwrap().sigma_serialize_bytes().unwrap();
        assert_eq!(&bytes[..2], vec![0u8, 8u8].as_slice());
    }

    #[test]
    fn test_constant_segregation() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(true),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(false), &expr).unwrap();
        let bytes = ergo_tree.sigma_serialize_bytes().unwrap();
        let parsed_expr = ErgoTree::sigma_parse_bytes(&bytes)
            .unwrap()
            .proposition()
            .unwrap();
        assert_eq!(parsed_expr, expr)
    }

    #[test]
    fn test_constant_len() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(false),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr).unwrap();
        assert_eq!(ergo_tree.constants_len().unwrap(), 1);
    }

    #[test]
    fn test_get_constant() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(false),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr).unwrap();
        assert_eq!(ergo_tree.constants_len().unwrap(), 1);
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), false.into());
    }

    #[test]
    fn test_set_constant() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(false),
        });
        let ergo_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &expr).unwrap();
        let new_ergo_tree = ergo_tree.with_constant(0, true.into()).unwrap();
        assert_eq!(new_ergo_tree.get_constant(0).unwrap().unwrap(), true.into());
    }

    #[test]
    fn dex_t2tpool_parse() {
        let base16_str = "19a3030f0400040204020404040404060406058080a0f6f4acdbe01b058080a0f6f4acdbe01b050004d00f0400040005000500d81ad601b2a5730000d602e4c6a70405d603db63087201d604db6308a7d605b27203730100d606b27204730200d607b27203730300d608b27204730400d609b27203730500d60ab27204730600d60b9973078c720602d60c999973088c720502720bd60d8c720802d60e998c720702720dd60f91720e7309d6108c720a02d6117e721006d6127e720e06d613998c7209027210d6147e720d06d615730ad6167e721306d6177e720c06d6187e720b06d6199c72127218d61a9c72167218d1edededededed93c27201c2a793e4c672010405720292c17201c1a793b27203730b00b27204730c00938c7205018c720601ed938c7207018c720801938c7209018c720a019593720c730d95720f929c9c721172127e7202069c7ef07213069a9c72147e7215067e9c720e720206929c9c721472167e7202069c7ef0720e069a9c72117e7215067e9c721372020695ed720f917213730e907217a19d721972149d721a7211ed9272199c7217721492721a9c72177211";
        let tree_bytes = base16::decode(base16_str.as_bytes()).unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();
        dbg!(&tree);
        let header = tree.parsed_tree().unwrap().header.clone();
        assert!(header.has_size());
        assert!(header.is_constant_segregation());
        assert_eq!(header.version(), &ErgoTreeVersion::V1);
        let new_tree = tree
            .with_constant(7, 1i64.into())
            .unwrap()
            .with_constant(8, 2i64.into())
            .unwrap();
        assert_eq!(new_tree.get_constant(7).unwrap().unwrap(), 1i64.into());
        assert_eq!(new_tree.get_constant(8).unwrap().unwrap(), 2i64.into());
        assert!(new_tree.sigma_serialize_bytes().unwrap().len() > 1);
    }

    #[test]
    fn parse_invalid_677() {
        // also see https://github.com/ergoplatform/sigma-rust/issues/587
        let base16_str = "cd07021a8e6f59fd4a";
        let tree_bytes = base16::decode(base16_str.as_bytes()).unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();
        dbg!(&tree);
        assert_eq!(tree.sigma_serialize_bytes().unwrap(), tree_bytes);
        assert_eq!(
            tree,
            ErgoTree::Unparsed {
                tree_bytes,
                error: ErgoTreeRootParsingError::NonConsumedBytes.into()
            }
        );
    }

    #[test]
    fn parse_invalid_tree_extra_bytes() {
        let valid_ergo_tree_hex =
            "0008cd02a706374307f3038cb2f16e7ae9d3e29ca03ea5333681ca06a9bd87baab1164bc";
        // extra bytes at the end will be left unparsed
        let invalid_ergo_tree_with_extra_bytes = format!("{}aaaa", valid_ergo_tree_hex);
        let bytes = base16::decode(invalid_ergo_tree_with_extra_bytes.as_bytes()).unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        dbg!(&tree);
        assert_eq!(tree.sigma_serialize_bytes().unwrap(), bytes);
        assert_eq!(
            tree,
            ErgoTree::Unparsed {
                tree_bytes: bytes,
                error: ErgoTreeRootParsingError::NonConsumedBytes.into()
            }
        );
    }
}
