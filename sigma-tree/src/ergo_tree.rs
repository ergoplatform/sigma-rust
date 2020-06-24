//! ErgoTree
use crate::{
    ast::{Constant, Expr},
    types::SType,
};
use io::Cursor;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::{peekable_reader::PeekableReader, vlq_encode};
use std::io;
use std::{convert::TryFrom, rc::Rc};
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

/** The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
#[allow(dead_code)]
pub struct ErgoTree {
    header: ErgoTreeHeader,
    constants: Vec<Constant>,
    root: Rc<Expr>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct ErgoTreeHeader(u8);

impl ErgoTree {
    const DEFAULT_HEADER: ErgoTreeHeader = ErgoTreeHeader(0);

    // TODO: move to Into and From implementations
    /// get Expr out of ErgoTree
    pub fn proposition(&self) -> Rc<Expr> {
        self.root.clone()
    }

    /// build ErgoTree from an Expr
    pub fn from_proposition(expr: Rc<Expr>) -> ErgoTree {
        match &*expr {
            Expr::Const(c) if c.tpe == SType::SSigmaProp => ErgoTree {
                header: ErgoTree::DEFAULT_HEADER,
                constants: Vec::new(),
                root: expr.clone(),
            },
            _ => panic!("not yet supported"),
        }
    }

    /// Serialized ErgoTree
    pub fn bytes(&self) -> Vec<u8> {
        // TODO: expensive, store in the struct?
        let mut data = Vec::new();
        // since it can only fail from IO error only it's safe to unwrap
        self.sigma_serialize(&mut data)
            .expect("serialization failed");
        data
    }
}

impl SigmaSerializable for ErgoTreeHeader {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u8(self.0)?;
        Ok(())
    }
    fn sigma_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let header = r.get_u8()?;
        Ok(ErgoTreeHeader(header))
    }
}

impl SigmaSerializable for ErgoTree {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        self.header.sigma_serialize(w)?;
        w.put_usize_as_u32(self.constants.len())?;
        assert!(
            self.constants.is_empty(),
            "separate constants serialization is not yet supported"
        );
        self.root.sigma_serialize(w)?;
        Ok(())
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let header = ErgoTreeHeader::sigma_parse(r)?;
        let constants_len = r.get_u32()?;
        assert!(
            constants_len == 0,
            "separate constants serialization is not yet supported"
        );
        let constants = Vec::new();
        let root = Expr::sigma_parse(r)?;
        Ok(ErgoTree {
            header,
            constants,
            root: Rc::new(root),
        })
    }
}

impl TryFrom<Vec<u8>> for ErgoTree {
    type Error = SerializationError;
    fn try_from(mut value: Vec<u8>) -> Result<Self, Self::Error> {
        let cursor = Cursor::new(&mut value[..]);
        let mut reader = PeekableReader::new(cursor);
        ErgoTree::sigma_parse(&mut reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast::ConstantVal, sigma_protocol::SigmaProp};
    use proptest::prelude::*;
    use sigma_ser::test_helpers::*;

    impl Arbitrary for ErgoTree {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<SigmaProp>())
                .prop_map(|p| {
                    ErgoTree::from_proposition(Rc::new(Expr::Const(Constant {
                        tpe: SType::SSigmaProp,
                        v: ConstantVal::SigmaProp(Box::new(p)),
                    })))
                })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ErgoTree>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&(v)), v];
        }
    }
}
