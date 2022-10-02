use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
// use crate::types::stuple::STuple;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/// Takes two collections as input and produces the concatenated collection (input + col2)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Append {
    /// Collection - First Parameter; first half of the combined collection
    pub input: Box<Expr>,
    /// Collection - Second Parameter; later half of the combined collection
    pub col_2: Box<Expr>,
}

impl Append {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, col_2: Expr) -> Result<Self, InvalidArgumentError> {
        match (input.post_eval_tpe(), col_2.post_eval_tpe()) {
            (SType::SColl(x), SType::SColl(y)) => {
                if x == y {
                    Ok(Append{input: input.into(), col_2: col_2.into()})
                } else {
                    Err(InvalidArgumentError(format!(
                        "Expected Append input and col_2 collection to have the same types; got input={0:?} col_2={1:?}",
                        x, y)))
                }
            }
            (SType::SColl(_), _) => {
                Err(InvalidArgumentError(format!(
                    "Expected Append col_2 param to be a collection; got col_2={:?}", col_2.tpe())))
            }
            (_, SType::SColl(_)) => {
                Err(InvalidArgumentError(format!(
                    "Expected Append input param to be a collection; got input={:?}", input.tpe())))   
            },
            (_, _) => {
                Err(InvalidArgumentError(format!(
                    "Expected Append input and col_2 param to be a collection; got input={:?} col_2={:?}", input.tpe(), col_2.tpe())))   
            }
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        // Type is supposed to be the same on input and col_2
        // Append::new checks types but later modifications are unchecked
        // return type of input collection
        self.input.tpe()
    }
}

impl HasStaticOpCode for Append {
    const OP_CODE: OpCode = OpCode::APPEND;
}

impl SigmaSerializable for Append {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.input.sigma_serialize(w)?;
        self.col_2.sigma_serialize(w)?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let input = Expr::sigma_parse(r)?;
        let col_2 = Expr::sigma_parse(r)?;
        Ok(Append::new(input, col_2)?)
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for Append {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        if name == "Append" {
            let content;
            let _paren = syn::parenthesized!(content in input);
            let input = Box::new(content.parse()?);
            let col_2 = Box::new(content.parse()?);

            Ok(Append { input, col_2 })
        } else {
            Err(syn::Error::new_spanned(name, "Expected `Append`"))
        }
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for Append {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let input = &*self.input;
        let col_2 = &*self.col_2;
        tokens.extend(quote::quote! {
            ergotree_ir::mir::coll_append::Append { input: Box::new(#input), col_2: Box::new(#col_2) }
        })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    impl Arbitrary for Append {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(SType::SBoolean.into()),
                    depth: 1,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(SType::SBoolean.into()),
                    depth: 1,
                }),
            )
                .prop_map(|(input, col_2)| Self {
                    input: input.into(),
                    col_2: col_2.into(),
                })
                .boxed()
        }
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<Append>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
