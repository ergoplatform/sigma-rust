//! Lookup in AVL tree
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::mir::expr::InvalidArgumentError;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::{SigmaParsingError, SigmaSerializable, SigmaSerializeResult};

/// Lookup in AVL tree
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
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(tree: Expr, key: Expr, proof: Expr) -> Result<Self, InvalidArgumentError> {
        tree.check_post_eval_tpe(&SType::SAvlTree)?;
        key.check_post_eval_tpe(&SType::SColl(SType::SByte.into()))?;
        proof.check_post_eval_tpe(&SType::SColl(SType::SByte.into()))?;
        Ok(TreeLookup {
            tree: tree.into(),
            key: key.into(),
            proof: proof.into(),
        })
    }

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

#[allow(clippy::unwrap_used)]
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
                .prop_map(|(tree, key, proof)| TreeLookup::new(tree, key, proof).unwrap())
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
