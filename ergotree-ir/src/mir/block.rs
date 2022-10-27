//! Block of statements ending with an expression

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::stype::SType;

/** Block of statements ending with an expression
 * The order of ValDefs in the block is used to assign ids to ValUse(id) nodes
 * For all i: items(i).id == {number of ValDefs preceded in a graph} with respect to topological order.
 * Specific topological order doesn't really matter, what is important is to preserve semantic linkage
 * between ValUse(id) and ValDef with the corresponding id.
 * This convention allow to valid serializing ids because we always serializing and deserializing
 * in a fixed well defined order.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockValue {
    /// Statements
    pub items: Vec<Expr>,
    /// Resulting expr
    pub result: Box<Expr>,
}

impl BlockValue {
    /// Type
    pub fn tpe(&self) -> SType {
        self.result.tpe()
    }
}

impl HasStaticOpCode for BlockValue {
    const OP_CODE: OpCode = OpCode::BLOCK_VALUE;
}

impl SigmaSerializable for BlockValue {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.items.sigma_serialize(w)?;
        self.result.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let items = Vec::<Expr>::sigma_parse(r)?;
        let result = Expr::sigma_parse(r)?;
        Ok(BlockValue {
            items,
            result: Box::new(result),
        })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for BlockValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = vec![];
        let ident: syn::Ident = input.parse()?;
        if ident != "Vector" && ident != "Array" {
            return Err(syn::Error::new_spanned(
                ident.clone(),
                format!(
                    "BlockValue(..): expected `Vector` or `Array` Ident, got {:?}",
                    ident
                ),
            ));
        }
        let content;
        let _paren = syn::parenthesized!(content in input);

        loop {
            let expr: Expr = content.parse()?;
            items.push(expr);
            if !content.peek(syn::Token![,]) {
                break;
            }
            let _comma: syn::Token![,] = content.parse()?;
        }
        let _comma: syn::Token![,] = input.parse()?;
        let result = input.parse()?;
        Ok(BlockValue { items, result })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for BlockValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let items = self.items.clone();
        let result = *self.result.clone();
        tokens.extend(quote::quote! {
            ergotree_ir::mir::block::BlockValue {
                items: vec![#( #items),*],
                result: Box::new(#result),
            }
        })
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for BlockValue {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), vec(any::<Expr>(), 0..10))
                .prop_map(|(result, items)| Self {
                    items,
                    result: Box::new(result),
                })
                .boxed()
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
pub mod tests {
    use crate::mir::block::BlockValue;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(block in any::<BlockValue>()) {
            let e = Expr::BlockValue(block);
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}
