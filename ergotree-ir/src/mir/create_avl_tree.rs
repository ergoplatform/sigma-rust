//! Create an AVL tree

use alloc::boxed::Box;
use alloc::sync::Arc;

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::mir::expr::InvalidArgumentError;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::{SigmaParsingError, SigmaSerializable, SigmaSerializeResult};
use crate::traversable::impl_traversable_expr;
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
    /// Expression of type SOption[SInt] yielding the optional value-length.
    /// Matches Scala's `valueLengthOpt: Value[SIntOption]` shape.
    pub value_length: Box<Expr>,
}

impl CreateAvlTree {
    /// Creates new AVL Tree
    pub fn new(
        flags: Expr,
        digest: Expr,
        key_length: Expr,
        value_length: Expr,
    ) -> Result<Self, InvalidArgumentError> {
        flags.check_post_eval_tpe(&SType::SByte)?;
        digest.check_post_eval_tpe(&SType::SColl(Arc::new(SType::SByte)))?;
        key_length.check_post_eval_tpe(&SType::SInt)?;
        value_length.check_post_eval_tpe(&SType::SOption(Arc::new(SType::SInt)))?;

        Ok(Self {
            flags: flags.into(),
            digest: digest.into(),
            key_length: key_length.into(),
            value_length: value_length.into(),
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
        let value_length = Expr::sigma_parse(r)?;
        Ok(Self::new(flags, digest, key_length, value_length)?)
    }

    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.flags.sigma_serialize(w)?;
        self.digest.sigma_serialize(w)?;
        self.key_length.sigma_serialize(w)?;
        self.value_length.sigma_serialize(w)
    }
}

impl_traversable_expr!(CreateAvlTree, boxed flags, boxed digest, boxed key_length, boxed value_length);

#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::constant::{Constant, Literal};
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

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
                    tpe: SType::SColl(Arc::new(SType::SByte)),
                    depth: 0,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
                proptest::option::of(any::<i32>()),
            )
                .prop_map(|(flags, digest, key_length, value_length_opt)| {
                    let inner_literal = value_length_opt.map(|v| Box::new(Literal::Int(v)));
                    let value_length: Expr = Constant {
                        tpe: SType::SOption(Arc::new(SType::SInt)),
                        v: Literal::Opt(inner_literal),
                    }
                    .into();
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
