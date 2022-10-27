use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::stype::SType;

use super::expr::Expr;

extern crate derive_more;
use derive_more::From;

use crate::has_opcode::HasStaticOpCode;
#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;

/// Variable id
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, From)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub struct ValId(pub u32);

impl ValId {
    pub(crate) fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> std::io::Result<()> {
        w.put_u32(self.0)
    }

    pub(crate) fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let id = r.get_u32()?;
        Ok(ValId(id))
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for ValId {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id: syn::LitInt = input.parse()?;
        let value = id.base10_parse::<u32>()?;
        Ok(ValId(value))
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for ValId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let val_id = self.0;
        tokens.extend(quote::quote! { ergotree_ir::mir::val_def::ValId(#val_id) })
    }
}

/** IR node for let-bound expressions `let x = rhs` which is ValDef.
 * These nodes are used to represent ErgoTrees after common sub-expression elimination.
 * This representation is more compact in serialized form.
 * @param id unique identifier of the variable in the current scope. */
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct ValDef {
    /// Variable id
    pub id: ValId,
    /// Expr, bound to the variable
    pub rhs: Box<Expr>,
}

impl ValDef {
    /// Type
    pub fn tpe(&self) -> SType {
        self.rhs.tpe()
    }
}

impl HasStaticOpCode for ValDef {
    const OP_CODE: OpCode = OpCode::VAL_DEF;
}

impl SigmaSerializable for ValDef {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.id.sigma_serialize(w)?;
        self.rhs.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let id = ValId::sigma_parse(r)?;
        let rhs = Expr::sigma_parse(r)?;
        r.val_def_type_store().insert(id, rhs.tpe());
        Ok(ValDef {
            id,
            rhs: Box::new(rhs),
        })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for ValDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id = input.parse()?;
        let _comma: syn::Token![,] = input.parse()?;
        let list_ident: syn::Ident = input.parse()?;
        if list_ident != "List" {
            return Err(syn::Error::new_spanned(
                list_ident.clone(),
                format!("Expected `List` ident, got {}", list_ident),
            ));
        }
        let _content;
        let _paren = syn::parenthesized!(_content in input);
        let _comma: syn::Token![,] = input.parse()?;
        let rhs = input.parse()?;
        Ok(Self { id, rhs })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for ValDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;
        let id = self.id;
        let rhs = *self.rhs.clone();
        tokens.extend(quote! {
            ergotree_ir::mir::val_def::ValDef { id: #id, rhs: Box::new(#rhs)}
        });
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ValDef>()) {
            let e = Expr::ValDef(v);
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
