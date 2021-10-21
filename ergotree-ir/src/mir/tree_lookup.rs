//! Convert SLong to byte array
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::{SigmaParsingError, SigmaSerializable, SigmaSerializeResult};

/// Tree lookup by key
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TreeLookup {
    /// Value of type SAvlTree
    pub tree: Box<Expr>,
    /// Byte array with SColl(SByte) expr type
    pub key: Box<Expr>,
    /// Byte array with SColl(SByte) expr type
    pub proof: Box<Expr>,
}

impl TreeLookup {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SOption(SType::SColl(SType::SByte.into()).into())
    }
}

impl HasStaticOpCode for TreeLookup {
    const OP_CODE: OpCode = OpCode::AVT_TREE_GET;
}

impl SigmaSerializable for TreeLookup {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.tree.sigma_serialize(w)?;
        self.key.sigma_serialize(w)?;
        self.proof.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let tree = Expr::sigma_parse(r)?.into();
        let key = Expr::sigma_parse(r)?.into();
        let proof = Expr::sigma_parse(r)?.into();
        Ok(TreeLookup { tree, key, proof })
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for TreeLookup {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SAvlTree,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(Box::new(SType::SByte)),
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(Box::new(SType::SByte)),
                    depth: 0,
                }),
            )
                .prop_map(|(tree, key, proof)| TreeLookup {
                    tree: tree.into(),
                    key: key.into(),
                    proof: proof.into(),
                })
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<TreeLookup>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
