//! Create an AVL tree

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::mir::expr::InvalidArgumentError;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::{SigmaParsingError, SigmaSerializable, SigmaSerializeResult};
use crate::types::stype::SType;

/// Creates an AVL tree
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CreateAvlTree {
    /// Value of type Byte
    pub flags: Box<Expr>,
    /// Byte array with SColl(SByte) expr type
    pub digest: Box<Expr>,
    /// Value of type Int
    pub key_length: Box<Expr>,
    /// Optional value of type Int
    pub value_length: Option<Box<Expr>>,
}

impl CreateAvlTree {
    /// Creates new AVL Tree
    pub fn new(
        flags: Expr,
        digest: Expr,
        key_length: Expr,
        value_length: Option<Box<Expr>>,
    ) -> Result<Self, InvalidArgumentError> {
        flags.check_post_eval_tpe(&SType::SByte)?;
        digest.check_post_eval_tpe(&SType::SColl(Box::new(SType::SByte)))?;
        key_length.check_post_eval_tpe(&SType::SInt)?;
        if !value_length
            .clone()
            .map(|expr| expr.post_eval_tpe() == SType::SInt)
            .unwrap_or(true)
        {
            return Err(InvalidArgumentError(format!(
                "CreateAvlTree: expected value_length type to be SInt, got {0:?}",
                value_length
            )));
        }

        Ok(Self {
            flags: flags.into(),
            digest: digest.into(),
            key_length: key_length.into(),
            value_length,
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SAvlTree
    }
}

impl HasStaticOpCode for CreateAvlTree {
    const OP_CODE: OpCode = OpCode::AVL_TREE;
}

impl SigmaSerializable for CreateAvlTree {
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let flags = Expr::sigma_parse(r)?;
        let digest = Expr::sigma_parse(r)?;
        let key_length = Expr::sigma_parse(r)?;
        let value_length = Option::<Box<Expr>>::sigma_parse(r)?;
        Ok(Self::new(flags, digest, key_length, value_length)?)
    }

    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.flags.sigma_serialize(w)?;
        self.digest.sigma_serialize(w)?;
        self.key_length.sigma_serialize(w)?;
        self.value_length.sigma_serialize(w)
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;
    use proptest::result::Probability;

    impl Arbitrary for CreateAvlTree {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SByte,
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(Box::new(SType::SByte)),
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
                any_with::<Option<Box<Expr>>>((
                    Probability::default(),
                    ArbExprParams {
                        tpe: SType::SInt,
                        depth: 0,
                    },
                )),
            )
                .prop_map(|(flags, digest, key_length, value_length)| {
                    Self::new(flags, digest, key_length, value_length).unwrap()
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
        fn ser_roundtrip(v in any::<CreateAvlTree>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
