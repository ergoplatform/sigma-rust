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
    /// Expression of type `Option[Int]` (matches JVM `CreateAvlTreeSerializer` wire format)
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
    use alloc::sync::Arc;

    use crate::mir::expr::arbitrary::ArbExprParams;
    use crate::mir::get_var::GetVar;

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
                any::<u8>().prop_map(|var_id| {
                    GetVar {
                        var_id,
                        var_tpe: SType::SInt,
                    }
                    .into()
                }),
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
#[allow(clippy::panic, clippy::unwrap_used)]
mod proptests {
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

#[cfg(test)]
#[allow(clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::mir::get_var::GetVar;
    use crate::serialization::SigmaSerializable;
    use alloc::vec;

    #[test]
    fn wire_format_encodes_value_length_expr_without_extra_option_tag() {
        let flags: Expr = 0i8.into();
        let digest: Expr = vec![0u8; 33].into();
        let key_length: Expr = 32i32.into();
        let value_length: Expr = GetVar {
            var_id: 1,
            var_tpe: SType::SInt,
        }
        .into();
        let expr: Expr = CreateAvlTree::new(
            flags.clone(),
            digest.clone(),
            key_length.clone(),
            value_length.clone(),
        )
        .unwrap()
        .into();

        let bytes = expr.sigma_serialize_bytes().unwrap();
        let mut expected = vec![OpCode::AVL_TREE.value()];
        expected.extend(flags.sigma_serialize_bytes().unwrap());
        expected.extend(digest.sigma_serialize_bytes().unwrap());
        expected.extend(key_length.sigma_serialize_bytes().unwrap());
        expected.extend(value_length.sigma_serialize_bytes().unwrap());
        assert_eq!(bytes, expected);
        assert_eq!(Expr::sigma_parse_bytes(&bytes).unwrap(), expr);
    }
}
