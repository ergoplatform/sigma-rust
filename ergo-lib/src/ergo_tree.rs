//! ErgoTree
use crate::ast::constant::Constant;
use crate::ast::constant::TryExtractFromError;
use crate::ast::expr::Expr;
use crate::serialization::{
    sigma_byte_reader::{SigmaByteRead, SigmaByteReader},
    sigma_byte_writer::{SigmaByteWrite, SigmaByteWriter},
    SerializationError, SigmaSerializable,
};
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::types::stype::SType;
use io::{Cursor, Read};

use crate::serialization::constant_store::ConstantStore;
use sigma_ser::{peekable_reader::PeekableReader, vlq_encode};
use std::convert::TryFrom;
use std::io;
use std::rc::Rc;
use thiserror::Error;
use vlq_encode::ReadSigmaVlqExt;

#[derive(PartialEq, Eq, Debug, Clone)]
struct ParsedTree {
    constants: Vec<Constant>,
    root: Result<Rc<Expr>, ErgoTreeRootParsingError>,
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

    /// Reasonable limit for the number of constants allowed in the ErgoTree
    pub const MAX_CONSTANTS_COUNT: usize = 4096;

    /// get Expr out of ErgoTree
    pub fn proposition(&self) -> Result<Rc<Expr>, ErgoTreeParsingError> {
        let root = self
            .tree
            .clone()
            .map_err(ErgoTreeParsingError::TreeParsingError)
            .and_then(|t| t.root.map_err(ErgoTreeParsingError::RootParsingError))?;
        if self.header.is_constant_segregation() {
            let mut data = Vec::new();
            let mut cs = ConstantStore::empty();
            let mut w = SigmaByteWriter::new(&mut data, Some(&mut cs));
            root.sigma_serialize(&mut w).unwrap();
            let cursor = Cursor::new(&mut data[..]);
            let pr = PeekableReader::new(cursor);
            let mut sr = SigmaByteReader::new_with_substitute_placeholders(
                pr,
                ConstantStore::new(self.tree.clone().unwrap().constants),
            );
            let parsed_expr = Expr::sigma_parse(&mut sr).unwrap();
            // todo!("substitute placeholders: {:?}", self.tree);
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
        let mut cs = ConstantStore::empty();
        let mut w = SigmaByteWriter::new(&mut data, Some(&mut cs));
        expr.sigma_serialize(&mut w).unwrap();
        let cursor = Cursor::new(&mut data[..]);
        let pr = PeekableReader::new(cursor);
        let constants = cs.get_all();
        let new_cs = ConstantStore::new(constants.clone());
        let mut sr = SigmaByteReader::new(pr, new_cs);
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
            constants
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

    fn sigma_parse_bytes(mut bytes: Vec<u8>) -> Result<Self, SerializationError> {
        let cursor = Cursor::new(&mut bytes[..]);
        let mut r = SigmaByteReader::new(PeekableReader::new(cursor), ConstantStore::empty());
        let header = ErgoTreeHeader::sigma_parse(&mut r)?;
        let constants = if header.is_constant_segregation() {
            let constants_len = r.get_u32()?;
            if constants_len as usize > ErgoTree::MAX_CONSTANTS_COUNT {
                return Err(SerializationError::ValueOutOfBounds(
                    "too many constants".to_string(),
                ));
            }
            let mut constants = Vec::with_capacity(constants_len as usize);
            for _ in 0..constants_len {
                match Constant::sigma_parse(&mut r) {
                    Ok(c) => constants.push(c),
                    Err(_) => {
                        return Ok(ErgoTree {
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
            constants
        } else {
            vec![]
        };
        let mut rest_of_the_bytes = Vec::new();
        let _ = r.read_to_end(&mut rest_of_the_bytes);
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

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::ast::value::Value;
    use crate::chain;
    use crate::chain::Base16DecodedBytes;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
    use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
    use crate::sigma_protocol::sigma_boolean::SigmaProp;
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

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ErgoTree>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&(v)), v];
        }
    }

    #[test]
    fn deserialization_non_parseable_tree_ok() {
        // constants length is set, invalid constant
        assert!(ErgoTree::sigma_parse_bytes(vec![
            ErgoTreeHeader::CONSTANT_SEGREGATION_FLAG,
            1,
            0,
            99,
            99
        ])
        .is_ok());
    }

    #[test]
    fn deserialization_non_parseable_root_ok() {
        // no constant segregation, Expr is invalid
        assert!(ErgoTree::sigma_parse_bytes(vec![0, 0, 1]).is_ok());
    }

    #[test]
    fn test_constant_segregation_header_flag_support() {
        let encoder = chain::address::AddressEncoder::new(chain::address::NetworkPrefix::Mainnet);
        let address = encoder
            .parse_address_from_str("9hzP24a2q8KLPVCUk7gdMDXYc7vinmGuxmLp5KU7k9UwptgYBYV")
            .unwrap();

        let contract = chain::contract::Contract::pay_to_address(&address).unwrap();
        let bytes = &contract.ergo_tree().sigma_serialize_bytes();
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
        let parsed_expr = ErgoTree::sigma_parse_bytes(bytes)
            .unwrap()
            .proposition()
            .unwrap();
        assert_eq!(*parsed_expr, expr)
    }
}
