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

use crate::serialization::constant_store::ConstantStore;
use sigma_ser::peekable_reader::PeekableReader;
use std::convert::TryFrom;
use std::io;
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
            Err(e) => e.bytes, // if tree was failed to parse we already have it's bytes
        }
    }
}

/** The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoTree {
    header: ErgoTreeHeader,
    tree: Result<ParsedTree, ErgoTreeConstantsParsingError>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct ErgoTreeHeader(u8);

impl ErgoTreeHeader {
    const CONSTANT_SEGREGATION_FLAG: u8 = 0x10;

    pub fn is_constant_segregation(&self) -> bool {
        self.0 & ErgoTreeHeader::CONSTANT_SEGREGATION_FLAG != 0
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
    pub bytes: Vec<u8>,
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

impl ErgoTree {
    const DEFAULT_HEADER: ErgoTreeHeader = ErgoTreeHeader(0);

    fn sigma_parse_tree_bytes<R: SigmaByteRead>(
        r: &mut R,
        is_constant_segregation: bool,
    ) -> Result<(Vec<Constant>, Vec<u8>), SerializationError> {
        let constants = if is_constant_segregation {
            ErgoTree::sigma_parse_constants(r)?
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
            let pr = PeekableReader::new(cursor);
            let mut sr = SigmaByteReader::new_with_substitute_placeholders(
                pr,
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

    /// Build ErgoTree using expr as is, without constants segregated
    pub fn without_segregation(expr: Expr) -> ErgoTree {
        ErgoTree {
            header: ErgoTree::DEFAULT_HEADER,
            tree: Ok(ParsedTree {
                constants: Vec::new(),
                root: Ok(Rc::new(expr)),
            }),
        }
    }

    /// Build ErgoTree with constants segregated from expr
    pub fn with_segregation(expr: &Expr) -> ErgoTree {
        let mut data = Vec::new();
        let cs = ConstantStore::empty();
        let mut w = SigmaByteWriter::new(&mut data, Some(cs));
        #[allow(clippy::unwrap_used)]
        expr.sigma_serialize(&mut w).unwrap();
        #[allow(clippy::unwrap_used)]
        let constants = w.constant_store_mut_ref().unwrap().get_all();
        let cursor = Cursor::new(&mut data[..]);
        let pr = PeekableReader::new(cursor);
        let new_cs = ConstantStore::new(constants.clone());
        let mut sr = SigmaByteReader::new(pr, new_cs);
        #[allow(clippy::unwrap_used)]
        // if it was serialized, then we should deserialize it without error
        let parsed_expr = Expr::sigma_parse(&mut sr).unwrap();
        ErgoTree {
            header: ErgoTreeHeader(ErgoTreeHeader::CONSTANT_SEGREGATION_FLAG),
            tree: Ok(ParsedTree {
                constants,
                root: Ok(Rc::new(parsed_expr)),
            }),
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
                    ErgoTree::without_segregation(expr)
                }
                _ => ErgoTree::with_segregation(&expr),
            },
            _ => ErgoTree::with_segregation(&expr),
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
        self.header.sigma_serialize(w)?;
        match &self.tree {
            Ok(ParsedTree { constants, root }) => {
                if self.header.is_constant_segregation() {
                    w.put_usize_as_u32(constants.len())?;
                    constants.iter().try_for_each(|c| c.sigma_serialize(w))?;
                }
                match root {
                    Ok(expr) => expr.sigma_serialize(w)?,
                    Err(ErgoTreeRootParsingError { bytes, .. }) => w.write_all(&bytes[..])?,
                }
            }
            Err(ErgoTreeConstantsParsingError { bytes, .. }) => w.write_all(&bytes[..])?,
        }
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let header = ErgoTreeHeader::sigma_parse(r)?;
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

    fn sigma_parse_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        let cursor = Cursor::new(bytes);
        let mut r = SigmaByteReader::new(PeekableReader::new(cursor), ConstantStore::empty());
        let header = ErgoTreeHeader::sigma_parse(&mut r)?;
        if let Ok((constants, mut rest_of_the_bytes)) =
            ErgoTree::sigma_parse_tree_bytes(&mut r, header.is_constant_segregation())
        {
            let rest_of_the_bytes_copy = rest_of_the_bytes.clone();
            let mut new_r = SigmaByteReader::new(
                PeekableReader::new(Cursor::new(&mut rest_of_the_bytes[..])),
                ConstantStore::new(constants.clone()),
            );
            match Expr::sigma_parse(&mut new_r) {
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
                            bytes: rest_of_the_bytes_copy,
                            error: err,
                        }),
                    }),
                }),
            }
        } else {
            Ok(ErgoTree {
                header,
                tree: Err(ErgoTreeConstantsParsingError {
                    bytes: bytes[1..].to_vec(),
                    error: SerializationError::NotImplementedYet(
                        "not all constant types serialization is supported".to_string(),
                    ),
                }),
            })
        }
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

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for ErgoTree {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                // make sure that P2PK tree is included
                any::<ProveDlog>().prop_map(|p| ErgoTree::from(Expr::Const(p.into()))),
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
            let mut data = Vec::new();
            let mut w = SigmaByteWriter::new(&mut data, None);
            v.sigma_serialize(&mut w).expect("serialization failed");
            let cursor = Cursor::new(&mut data[..]);
            let pr = PeekableReader::new(cursor);
            let mut sr = SigmaByteReader::new(pr, ConstantStore::empty());
            let res = ErgoTree::sigma_parse(&mut sr).expect("parse failed");
            prop_assert_eq!(&res.template_bytes().unwrap(), &v.template_bytes().unwrap());
            prop_assert_eq![res, v];
        }
    }

    #[test]
    fn deserialization_non_parseable_tree_ok() {
        // constants length is set, invalid constant
        assert!(ErgoTree::sigma_parse_bytes(&[
            ErgoTreeHeader::CONSTANT_SEGREGATION_FLAG,
            1,
            0,
            99,
            99
        ])
        .is_ok());
    }

    #[test]
    fn serialization_non_parseable_tree_ok() {
        // constants length is set, invalid constant
        let original = &[ErgoTreeHeader::CONSTANT_SEGREGATION_FLAG, 1, 0, 99, 99];
        let tree = ErgoTree::sigma_parse_bytes(original).unwrap();
        let bytes = tree.sigma_serialize_bytes();
        assert_eq!(bytes, original);
        assert!(tree.template_bytes().is_err());
    }

    #[test]
    fn deserialization_non_parseable_root_ok() {
        // no constant segregation, Expr is invalid
        assert!(ErgoTree::sigma_parse_bytes(&[0, 0, 1]).is_ok());
    }

    #[test]
    fn serialization_non_parseable_root_ok() {
        // no constant segregation, Expr is invalid
        let original = &[0, 0, 1];
        let tree = ErgoTree::sigma_parse_bytes(original).unwrap();
        // serialization should return bytes that were failed to parse
        let bytes = tree.sigma_serialize_bytes();
        assert_eq!(bytes, original);
        assert_eq!(tree.template_bytes().unwrap(), [0, 1]); // header byte is skipped
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
        let ergo_tree = ErgoTree::with_segregation(&expr);
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
        let ergo_tree = ErgoTree::with_segregation(&expr);
        assert_eq!(ergo_tree.constants_len().unwrap(), 1);
    }

    #[test]
    fn test_get_constant() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Value::Boolean(false),
        });
        let ergo_tree = ErgoTree::with_segregation(&expr);
        assert_eq!(ergo_tree.constants_len().unwrap(), 1);
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), false.into());
    }

    #[test]
    fn test_set_constant() {
        let expr = Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Value::Boolean(false),
        });
        let mut ergo_tree = ErgoTree::with_segregation(&expr);
        assert_eq!(
            ergo_tree.set_constant(0, true.into()).unwrap().unwrap(),
            false.into()
        );
        assert_eq!(ergo_tree.get_constant(0).unwrap().unwrap(), true.into());
    }
}
