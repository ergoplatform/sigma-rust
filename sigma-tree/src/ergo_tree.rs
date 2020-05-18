//! ErgoTree
use crate::{
    ast::{Constant, Expr},
    types::SType,
};
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

/** The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
 */
// #[derive(PartialEq, Debug)]
pub struct ErgoTree {
    header: ErgoTreeHeader,
    constants: Vec<Constant>,
    root: Box<Expr>,
}

struct ErgoTreeHeader(u8);

impl ErgoTree {
    const DEFAULT_HEADER: ErgoTreeHeader = ErgoTreeHeader(0);

    /// get expr out of ErgoTree
    pub fn to_proposition(&self) -> Box<Expr> {
        self.root
    }

    pub fn from_proposition(expr: Box<Expr>) -> ErgoTree {
        match *expr {
            Expr::Const(c) if c.tpe == SType::SSigmaProp => ErgoTree {
                header: ErgoTree::DEFAULT_HEADER,
                constants: Vec::new(),
                root: expr,
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
        Ok(ErgoTree {})
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
