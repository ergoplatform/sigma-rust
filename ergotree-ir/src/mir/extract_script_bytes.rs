use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::unary_op::OneArgOp;
use super::unary_op::OneArgOpTryBuild;
use crate::has_opcode::HasStaticOpCode;

/// Serialized box guarding script
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtractScriptBytes {
    /// Box, type of SBox
    pub input: Box<Expr>,
}

impl ExtractScriptBytes {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(SType::SByte.into())
    }
}

impl HasStaticOpCode for ExtractScriptBytes {
    const OP_CODE: OpCode = OpCode::EXTRACT_SCRIPT_BYTES;
}

impl OneArgOp for ExtractScriptBytes {
    fn input(&self) -> &Expr {
        &self.input
    }
}

impl OneArgOpTryBuild for ExtractScriptBytes {
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SBox)?;
        Ok(ExtractScriptBytes {
            input: input.into(),
        })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for ExtractScriptBytes {
    fn parse(buf: syn::parse::ParseStream) -> syn::Result<Self> {
        let input: Box<Expr> = buf.parse()?;
        Ok(Self { input })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for ExtractScriptBytes {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let input = *self.input.clone();
        tokens.extend(
            quote::quote! { ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes{
                 input: Box::new(#input),
            }},
        )
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = ExtractScriptBytes {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
