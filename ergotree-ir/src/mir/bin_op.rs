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
    /// Modulo
    Modulo,
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for ArithOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;
        tokens.extend(match self {
            ArithOp::Plus => quote! { ergotree_ir::mir::bin_op::ArithOp::Plus },
            ArithOp::Minus => quote! { ergotree_ir::mir::bin_op::ArithOp::Minus },
            ArithOp::Multiply => quote! { ergotree_ir::mir::bin_op::ArithOp::Multiply },
            ArithOp::Divide => quote! { ergotree_ir::mir::bin_op::ArithOp::Divide },
            ArithOp::Max => quote! { ergotree_ir::mir::bin_op::ArithOp::Max },
            ArithOp::Min => quote! { ergotree_ir::mir::bin_op::ArithOp::Min },
            ArithOp::Modulo => quote! { ergotree_ir::mir::bin_op::ArithOp::Modulo },
        });
    }
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
            ArithOp::Modulo => OpCode::MODULO,
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
    /// Greater or equal
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

#[cfg(feature = "ergotree-proc-macro")]
/// Given name of a binary op, parse an instance of `BinOp`
pub fn parse_bin_op(op_name: &syn::Ident, input: syn::parse::ParseStream) -> syn::Result<BinOp> {
    match op_name.to_string().as_str() {
        "ArithOp" => {
            let left: Box<Expr> = input.parse()?;
            let _comma: syn::Token![,] = input.parse()?;
            let right: Box<Expr> = input.parse()?;
            let _comma: syn::Token![,] = input.parse()?;
            let kind = extract_arithmetic_bin_op_kind(input)?;
            Ok(BinOp { kind, left, right })
        }
        _ => Err(syn::Error::new_spanned(
            op_name.clone(),
            "Unknown `BinOp` variant name",
        )),
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for BinOp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;
        let left = *self.left.clone();
        let right = *self.right.clone();
        tokens.extend(match self.kind {
            BinOpKind::Arith(a) => {
                quote! {
                    ergotree_ir::mir::bin_op::BinOp {
                        left: Box::new(#left),
                        right: Box::new(#right),
                        kind: ergotree_ir::mir::bin_op::BinOpKind::Arith(#a),
                    }
                }
            }
            _ => todo!(),
        });
    }
}
#[cfg(feature = "ergotree-proc-macro")]
/// Converts `OpCode @@ x` into an instance of `BinOpKind::Arith`.
fn extract_arithmetic_bin_op_kind(buf: syn::parse::ParseStream) -> Result<BinOpKind, syn::Error> {
    let ident: syn::Ident = buf.parse()?;
    if ident == "OpCode" {
        let _at: syn::Token![@] = buf.parse()?;
        let _at: syn::Token![@] = buf.parse()?;
        let content;
        let _paren = syn::parenthesized!(content in buf);
        let id: syn::LitInt = content.parse()?;
        let scala_op_code = id.base10_parse::<i32>()?;
        let _dot: syn::Token![.] = content.parse()?;
        let as_byte_ident: syn::Ident = content.parse()?;
        if as_byte_ident != "toByte" {
            return Err(syn::Error::new_spanned(
                as_byte_ident.clone(),
                format!("Expected `asByte` Ident, got {}", as_byte_ident),
            ));
        }
        match OpCode::parse(scala_op_code as u8) {
            OpCode::PLUS => Ok(ArithOp::Plus.into()),
            OpCode::MINUS => Ok(ArithOp::Minus.into()),
            OpCode::MULTIPLY => Ok(ArithOp::Multiply.into()),
            OpCode::DIVISION => Ok(ArithOp::Divide.into()),
            OpCode::MAX => Ok(ArithOp::Max.into()),
            OpCode::MIN => Ok(ArithOp::Min.into()),
            OpCode::MODULO => Ok(ArithOp::Modulo.into()),
            _ => Err(syn::Error::new_spanned(ident, "Expected arithmetic opcode")),
        }
    } else {
        Err(syn::Error::new_spanned(
            ident.clone(),
            format!("Expected `OpCode` ident, got {} ", ident),
        ))
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
    use crate::mir::constant::Literal::Boolean;
    use crate::mir::expr::Expr;
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
