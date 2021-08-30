//! Operators in ErgoTree

use super::expr::Expr;
use crate::has_opcode::HasOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

extern crate derive_more;
use derive_more::From;

#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;

/// Operations for numerical types
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum ArithOp {
    /// Addition
    Plus,
    /// Subtraction
    Minus,
    /// Multiplication
    Multiply,
    /// Division
    Divide,
    /// Max of two values
    Max,
    /// Min of two values
    Min,
}

impl From<ArithOp> for OpCode {
    fn from(op: ArithOp) -> Self {
        match op {
            ArithOp::Plus => OpCode::PLUS,
            ArithOp::Minus => OpCode::MINUS,
            ArithOp::Multiply => OpCode::MULTIPLY,
            ArithOp::Divide => OpCode::DIVISION,
            ArithOp::Max => OpCode::MAX,
            ArithOp::Min => OpCode::MIN,
        }
    }
}

/// Relational operations
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum RelationOp {
    /// Equality
    Eq,
    /// Non-equality
    NEq,
    /// Greater of equal
    Ge,
    /// Greater than..
    Gt,
    /// Less or equal
    Le,
    /// Less then
    Lt,
}

impl From<RelationOp> for OpCode {
    fn from(op: RelationOp) -> Self {
        match op {
            RelationOp::Eq => OpCode::EQ,
            RelationOp::NEq => OpCode::NEQ,
            RelationOp::Ge => OpCode::GE,
            RelationOp::Gt => OpCode::GT,
            RelationOp::Le => OpCode::LE,
            RelationOp::Lt => OpCode::LT,
        }
    }
}

/// Logical operations
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum LogicalOp {
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Logical XOR
    Xor,
}

impl From<LogicalOp> for OpCode {
    fn from(op: LogicalOp) -> Self {
        match op {
            LogicalOp::And => OpCode::BIN_AND,
            LogicalOp::Or => OpCode::BIN_OR,
            LogicalOp::Xor => OpCode::BIN_XOR,
        }
    }
}

/// Bitwise operations
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum BitOp {
    /// Bitwise Or
    BitOr,
    /// Bitwise And
    BitAnd,
    /// Bitwise Xor
    BitXor,
}

impl From<BitOp> for OpCode {
    fn from(op: BitOp) -> Self {
        match op {
            BitOp::BitOr => OpCode::BIT_OR,
            BitOp::BitAnd => OpCode::BIT_AND,
            BitOp::BitXor => OpCode::BIT_XOR,
        }
    }
}

/// Binary operations
#[derive(PartialEq, Eq, Debug, Clone, Copy, From)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum BinOpKind {
    /// Arithmetic operations
    Arith(ArithOp),
    /// Relation operations (equality, comparison, etc.)
    Relation(RelationOp),
    /// Logical operations
    Logical(LogicalOp),
    /// Bitwise operations
    Bit(BitOp),
}

impl From<BinOpKind> for OpCode {
    fn from(op: BinOpKind) -> Self {
        match op {
            BinOpKind::Arith(o) => o.into(),
            BinOpKind::Relation(o) => o.into(),
            BinOpKind::Logical(o) => o.into(),
            BinOpKind::Bit(o) => o.into(),
        }
    }
}

/// Binary operation
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BinOp {
    /// Operation kind
    pub kind: BinOpKind,
    /// Left operand
    pub left: Box<Expr>,
    /// Right operand
    pub right: Box<Expr>,
}

impl BinOp {
    /// Type
    pub fn tpe(&self) -> SType {
        match self.kind {
            BinOpKind::Relation(_) => SType::SBoolean,
            BinOpKind::Arith(_) => self.left.tpe(),
            BinOpKind::Logical(_) => SType::SBoolean,
            BinOpKind::Bit(_) => self.left.tpe(),
        }
    }
}

impl HasOpCode for BinOp {
    fn op_code(&self) -> OpCode {
        self.kind.into()
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for BinOp {
        type Parameters = ArbExprParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            let numeric_binop = || -> BoxedStrategy<BinOp> {
                (
                    prop_oneof![
                        any::<ArithOp>().prop_map_into(),
                        any::<BitOp>().prop_map_into(),
                    ],
                    any_with::<Expr>(args.clone()),
                    any_with::<Expr>(args.clone()),
                )
                    .prop_map(|(kind, left, right)| BinOp {
                        kind,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                    .boxed()
            };

            match args.tpe {
                SType::SBoolean => prop_oneof![
                    (
                        any::<RelationOp>().prop_map_into(),
                        any_with::<Expr>(ArbExprParams {
                            tpe: SType::SAny,
                            depth: args.depth,
                        }),
                        any_with::<Expr>(ArbExprParams {
                            tpe: SType::SAny,
                            depth: args.depth,
                        }),
                    ),
                    (
                        any::<LogicalOp>().prop_map_into(),
                        any_with::<Expr>(ArbExprParams {
                            tpe: SType::SBoolean,
                            depth: args.depth,
                        }),
                        any_with::<Expr>(ArbExprParams {
                            tpe: SType::SBoolean,
                            depth: args.depth,
                        }),
                    )
                ]
                .prop_map(|(kind, left, right)| BinOp {
                    kind,
                    left: Box::new(left),
                    right: Box::new(right),
                })
                .boxed(),

                SType::SByte => numeric_binop(),
                SType::SShort => numeric_binop(),
                SType::SInt => numeric_binop(),
                SType::SLong => numeric_binop(),
                SType::SBigInt => numeric_binop(),

                _ => (
                    any::<BinOpKind>(),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SAny,
                        depth: args.depth,
                    }),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SAny,
                        depth: args.depth,
                    }),
                )
                    .prop_map(|(kind, left, right)| BinOp {
                        kind,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                    .boxed(),
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {

    use super::*;
    use crate::mir::constant::Constant;
    use crate::mir::expr::Expr;
    use crate::mir::value::Value::Boolean;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::serialization::SigmaSerializable;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<BinOp>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }

    // Test that binop with boolean literals serialized correctly
    #[test]
    fn regression_249() {
        let e = Expr::sigma_parse_bytes(&[0xed, 0x85, 0x03]);
        assert_eq!(
            e,
            Ok(Expr::BinOp(BinOp {
                kind: BinOpKind::Logical(LogicalOp::And,),
                left: Expr::Const(Constant {
                    tpe: SType::SBoolean,
                    v: Boolean(true),
                })
                .into(),
                right: Expr::Const(Constant {
                    tpe: SType::SBoolean,
                    v: Boolean(true),
                })
                .into(),
            }))
        );
    }
}
