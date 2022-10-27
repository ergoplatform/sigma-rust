use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

/// Logical NOT (inverts the input)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct LogicalNot {
    /// Input expr of SBoolean type
    pub input: Box<Expr>,
}

impl LogicalNot {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SBoolean
    }
}

impl HasStaticOpCode for LogicalNot {
    const OP_CODE: OpCode = OpCode::LOGICAL_NOT;
}

impl OneArgOp for LogicalNot {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl OneArgOpTryBuild for LogicalNot {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SBoolean)?;
        Ok(Self {
            input: input.into(),
        })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for LogicalNot {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input: Box<Expr> = input.parse()?;
        Ok(Self { input })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for LogicalNot {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let input = *self.input.clone();
        tokens.extend(quote::quote! { ergotree_ir::mir::logical_not::LogicalNot{
             input: Box::new(#input),
        }})
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for LogicalNot {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SBoolean,
                depth: args,
            })
            .prop_map(|input| Self {
                input: input.into(),
            })
            .boxed()
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<LogicalNot>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
