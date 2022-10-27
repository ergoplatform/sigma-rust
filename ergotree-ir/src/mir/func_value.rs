use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::sfunc::SFunc;
use crate::types::stype::SType;

use super::expr::Expr;
use super::val_def::ValId;

#[cfg(test)]
use proptest_derive::Arbitrary;

/// Argument parameter for the user-defined function [`FuncValue`]
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct FuncArg {
    /// Value id (defined with [`super::val_def::ValDef`])
    pub idx: ValId,
    /// Value type
    pub tpe: SType,
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for FuncArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _paren = syn::parenthesized!(content in input);
        let id: syn::LitInt = content.parse()?;
        let value = id.base10_parse::<u32>()?;
        let idx = ValId(value);
        let _comma: syn::Token![,] = content.parse()?;
        let tpe = content.parse()?;
        Ok(FuncArg { idx, tpe })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for FuncArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let idx = &self.idx;
        let tpe = &self.tpe;
        tokens.extend(quote::quote! {
            ergotree_ir::mir::func_value::FuncArg {
                idx: #idx,
                tpe: #tpe,
            }
        })
    }
}

impl SigmaSerializable for FuncArg {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.idx.sigma_serialize(w)?;
        self.tpe.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let idx = ValId::sigma_parse(r)?;
        let tpe = SType::sigma_parse(r)?;
        Ok(FuncArg { idx, tpe })
    }
}

/// User-defined function
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FuncValue {
    args: Vec<FuncArg>,
    body: Box<Expr>,
    tpe: SType,
}

impl FuncValue {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(args: Vec<FuncArg>, body: Expr) -> Self {
        let t_dom = args.iter().map(|fa| fa.tpe.clone()).collect();
        let t_range = body.tpe();
        let tpe = SType::SFunc(SFunc {
            t_dom,
            t_range: Box::new(t_range),
            tpe_params: vec![],
        });
        FuncValue {
            args,
            body: Box::new(body),
            tpe,
        }
    }

    /// Function arguments
    pub fn args(&self) -> &[FuncArg] {
        self.args.as_ref()
    }

    /// Function body
    pub fn body(&self) -> &Expr {
        &self.body
    }

    /// Type
    pub fn tpe(&self) -> SType {
        self.tpe.clone()
    }
}

impl HasStaticOpCode for FuncValue {
    const OP_CODE: OpCode = OpCode::FUNC_VALUE;
}

impl SigmaSerializable for FuncValue {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.args.sigma_serialize(w)?;
        self.body.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let args = Vec::<FuncArg>::sigma_parse(r)?;
        args.iter()
            .for_each(|a| r.val_def_type_store().insert(a.idx, a.tpe.clone()));
        let body = Expr::sigma_parse(r)?;
        Ok(FuncValue::new(args, body))
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for FuncValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = {
            let name: syn::Ident = input.parse()?;
            if name == "Vector" {
                let content;
                let _paren = syn::parenthesized!(content in input);
                let punctuated: syn::punctuated::Punctuated<FuncArg, syn::Token![,]> =
                    content.parse_terminated(FuncArg::parse)?;
                punctuated.into_iter().collect()
            } else {
                return Err(syn::Error::new_spanned(name, "Expected `Vector`"));
            }
        };
        let _comma: syn::Token![,] = input.parse()?;
        let body: Expr = input.parse()?;
        Ok(FuncValue::new(args, body))
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for FuncValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let args = &self.args;
        let body = &*self.body;
        //let tpe = &self.tpe;
        tokens.extend(
            quote::quote! { ergotree_ir::mir::func_value::FuncValue::new(
                 vec![#( #args),*],
                #body,
            )},
        )
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;

    use proptest::collection::vec;
    use proptest::prelude::*;

    use super::*;

    impl Arbitrary for FuncValue {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), vec(any::<FuncArg>(), 1..10))
                .prop_map(|(body, args)| Self::new(args, body))
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(func_value in any::<FuncValue>()) {
            let e = Expr::FuncValue(func_value);
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
