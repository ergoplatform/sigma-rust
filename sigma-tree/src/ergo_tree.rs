//! ErgoTree
#![allow(unused_imports)]
use crate::ast::Expr;
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::io;

/** The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
 */
#[derive(PartialEq, Debug)]
pub struct ErgoTree {}

impl ErgoTree {
    /// get value out of ErgoTree
    pub fn to_proposition(&self, _replace_constants: bool) -> Box<Expr> {
        todo!()
        // let c = ConstantNode {
        //     value: Box::new(CSigmaProp {
        //         sigma_tree: SigmaBoolean::ProveDlog(0),
        //     }) as Box<dyn SigmaProp>,
        //     tpe: SType::SSigmaProp,
        // };
        // Box::new(c)
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
