//! ErgoTree
use crate::{
    ast::{Constant, Expr},
    types::SType,
};
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;
use std::rc::Rc;

/** The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
 */
#[derive(PartialEq, Debug)]
#[allow(dead_code)]
pub struct ErgoTree {
    header: ErgoTreeHeader,
    constants: Vec<Constant>,
    root: Rc<Expr>,
}

#[derive(PartialEq, Debug)]
struct ErgoTreeHeader(u8);

impl ErgoTree {
    const DEFAULT_HEADER: ErgoTreeHeader = ErgoTreeHeader(0);

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
}

impl SigmaSerializable for ErgoTree {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, _: W) -> Result<(), io::Error> {
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: R) -> Result<Self, SerializationError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use sigma_ser::test_helpers::*;
    use sigma_testutil::generator::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ErgoTreeArb>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&(v.0)), v.0];
        }
    }
}
