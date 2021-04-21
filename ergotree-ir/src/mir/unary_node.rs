//! Node of AST that wraps single expression
use crate::mir::expr::{Expr, InvalidArgumentError};
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;
use std::marker::PhantomData;

/// Node of AST that only wraps single expression. Its semantics is determined by type tag `T`
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Unary<T> {
    /// Wrapped expression.
    pub input: Box<Expr>,
    /// Rust doesn't accept `T` as phantom type. phantom ony exists to please rustc.
    phantom: PhantomData<T>,
}

/// Trait which contains opcode and typing of AST node for Unary
pub trait TagOpCode {
    /// Op code of AST node
    const OP_CODE: OpCode;
    /// Type of unary node.
    fn res_tpe(expr: &Expr) -> SType;
    /// Check whether expression is valid argument. It's used during creation of unary node.
    fn check_arg(expr: &Expr) -> Result<(), InvalidArgumentError>;
}

impl<T: TagOpCode> Unary<T> {
    /// Create unary node. It checks whether argument has correct type.
    pub fn new(e: Expr) -> Result<Unary<T>, InvalidArgumentError> {
        T::check_arg(&e)?;
        Ok(Unary {
            input: e.into(),
            phantom: PhantomData,
        })
    }

    /// Op code for AST node
    pub const OP_CODE: OpCode = T::OP_CODE;

    /// Type of expression
    pub fn tpe(&self) -> SType {
        T::res_tpe(&self.input)
    }

    pub(crate) fn op_code(&self) -> OpCode {
        T::OP_CODE
    }
}

impl<T: TagOpCode> SigmaSerializable for Unary<T> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let r = Unary::<T>::new(input)?;
        Ok(r)
    }
}

/// Type tag for CalcBlake2b256
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TagCalcBlake2b256;

/// Calc Blake2b 256-bit hash
pub type CalcBlake2b256 = Unary<TagCalcBlake2b256>;

impl TagOpCode for TagCalcBlake2b256 {
    const OP_CODE: OpCode = OpCode::CALC_BLAKE2B256;
    fn res_tpe(_: &Expr) -> SType {
        SType::SColl(Box::new(SType::SByte))
    }
    fn check_arg(expr: &Expr) -> Result<(), InvalidArgumentError> {
        expr.check_post_eval_tpe(SType::SColl(Box::new(SType::SByte)))?;
        Ok(())
    }
}

/// Tag for OptionGet
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TagOptionGet;

/// Returns the Option's value or error if no value
pub type OptionGet = Unary<TagOptionGet>;

impl TagOpCode for TagOptionGet {
    const OP_CODE: OpCode = OpCode::OPTION_GET;
    fn res_tpe(expr: &Expr) -> SType {
        match expr.tpe() {
            SType::SOption(o) => *o,
            tpe => panic!(
                "expected OptionGet::input type to be SOption, got: {0:?}",
                &tpe
            ),
        }
    }
    fn check_arg(expr: &Expr) -> Result<(), InvalidArgumentError> {
        match expr.post_eval_tpe() {
            SType::SOption(_) => Ok(()),
            tpe => Err(InvalidArgumentError(format!(
                "expected OptionGet::input type to be SOption, got: {0:?}",
                tpe,
            ))),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Tag for SizeOf
pub struct TagSizeOf;
/// Collection size
pub type SizeOf = Unary<TagSizeOf>;

impl TagOpCode for TagSizeOf {
    const OP_CODE: OpCode = OpCode::SIZE_OF;
    fn res_tpe(_: &Expr) -> SType {
        SType::SInt
    }
    fn check_arg(expr: &Expr) -> Result<(), InvalidArgumentError> {
        match expr.post_eval_tpe() {
            SType::SColl(_) => Ok(()),
            tpe => Err(InvalidArgumentError(format!(
                "Expected SizeOf input to be SColl, got {0:?}",
                &tpe
            ))),
        }
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    pub trait UnaryExpr {
        fn gen_expr_tpe() -> SType;
    }

    impl<T: TagOpCode + std::fmt::Debug + UnaryExpr> Arbitrary for Unary<T> {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: T::gen_expr_tpe(),
                depth: 1,
            })
            .prop_map(|input| Self::new(input).unwrap())
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::unary_node::arbitrary::UnaryExpr;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::stype_param::STypeVar;
    use proptest::prelude::*;

    impl UnaryExpr for TagCalcBlake2b256 {
        fn gen_expr_tpe() -> SType {
            SType::SColl(SType::SByte.into())
        }
    }

    impl UnaryExpr for TagSizeOf {
        fn gen_expr_tpe() -> SType {
            SType::SColl(SType::STypeVar(STypeVar::t()).into())
        }
    }

    impl UnaryExpr for TagOptionGet {
        fn gen_expr_tpe() -> SType {
            SType::SOption(SType::SInt.into())
        }
    }

    proptest! {
        #[test]
        fn ser_roundtrip_blake2b256(v in any::<CalcBlake2b256>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

        #[test]
        fn ser_roundtrip_sizeof(v in any::<SizeOf>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

        #[test]
        fn ser_roundtrip_option_get(v in any::<SizeOf>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
