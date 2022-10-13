use std::convert::TryFrom;

#[cfg(feature = "ergotree-proc-macro")]
use crate::ergotree_proc_macro::extract_tpe_from_dot_typed;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::stuple::STuple;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/// Tuple field access index (1..=255)
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct TupleFieldIndex(u8);

/// Error for tuple index being out of bounds (1..=255)
#[derive(Debug)]
pub struct TupleFieldIndexOutBounds;

impl TryFrom<u8> for TupleFieldIndex {
    type Error = TupleFieldIndexOutBounds;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value >= 1 {
            Ok(TupleFieldIndex(value))
        } else {
            Err(TupleFieldIndexOutBounds)
        }
    }
}

impl TupleFieldIndex {
    /// Returns a zero-based index
    pub fn zero_based_index(&self) -> usize {
        (self.0 - 1) as usize
    }
}

impl SigmaSerializable for TupleFieldIndex {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.0)?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let field_index = r.get_u8()?;
        TupleFieldIndex::try_from(field_index).map_err(|_| {
            SigmaParsingError::ValueOutOfBounds(format!(
                "invalid tuple field index: {0}",
                field_index
            ))
        })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for TupleFieldIndex {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id: syn::LitInt = input.parse()?;
        let value = id.base10_parse::<u8>()?;
        Ok(Self(value))
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for TupleFieldIndex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field_index = self.0;
        tokens.extend(quote::quote! { ergotree_ir::mir::select_field::TupleFieldIndex::try_from(#field_index).unwrap() })
    }
}

/// Select a field of the tuple value
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SelectField {
    /// Tuple value
    pub input: Box<Expr>,
    /// 1-based tuple field index (input._1 has field_index of 1)
    pub field_index: TupleFieldIndex,
    /// Field type
    field_tpe: SType,
}

impl SelectField {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, field_index: TupleFieldIndex) -> Result<Self, InvalidArgumentError> {
        match input.tpe() {
            SType::STuple(STuple { items }) => {
                if let Some(field_tpe) = items.get(field_index.zero_based_index()) {
                    Ok(SelectField {
                        input: Box::new(input),
                        field_index,
                        field_tpe: field_tpe.clone(),
                    })
                } else {
                    Err(InvalidArgumentError(format!(
                        "SelectField field index is out of bounds {0:?}, tuple type: {1:?}",
                        field_index,
                        input.tpe()
                    )))
                }
            }
            tpe => Err(InvalidArgumentError(format!(
                "expected SelectField input type to be STuple, got {0:?}",
                tpe
            ))),
        }
    }
}

impl HasStaticOpCode for SelectField {
    const OP_CODE: OpCode = OpCode::SELECT_FIELD;
}

impl SelectField {
    /// Type
    pub fn tpe(&self) -> SType {
        self.field_tpe.clone()
    }
}

impl SigmaSerializable for SelectField {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.input.sigma_serialize(w)?;
        self.field_index.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let input = Expr::sigma_parse(r)?;
        let field_index = TupleFieldIndex::sigma_parse(r)?;
        Ok(SelectField::new(input, field_index)?)
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for SelectField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _dot: syn::Token![.] = input.parse()?;
        let name: syn::Ident = input.parse()?;
        if name == "typed" {
            let mut content;
            let _bracketed = syn::bracketed!(content in input);
            let extracted_type = extract_tpe_from_dot_typed(&content)?;
            let _paren = syn::parenthesized!(content in input);
            let input: Expr = content.parse()?;
            let _comma: syn::Token![,] = content.parse()?;
            let id: syn::LitInt = content.parse()?;
            let value = id.base10_parse::<u8>()?;
            let field_index = TupleFieldIndex::try_from(value).map_err(|_| {
                syn::Error::new_spanned(name.clone(), "Expected `field_index` >= 1")
            })?;
            let _dot: syn::Token![.] = content.parse()?;
            let _to_byte_ident: syn::Ident = content.parse()?;
            let sf = Self::new(input, field_index).map_err(|e| {
                syn::Error::new_spanned(name.clone(), format!("SelectField::new error: {}", e))
            })?;
            match extracted_type {
                crate::ergotree_proc_macro::ExtractedType::FullySpecified(field_tpe) => {
                    if sf.field_tpe != field_tpe {
                        Err(syn::Error::new_spanned(
                            name,
                            format!(
                                "Expected tuple field of type {:?}, got {:?} ",
                                field_tpe, sf.field_tpe
                            ),
                        ))
                    } else {
                        Ok(sf)
                    }
                }
                crate::ergotree_proc_macro::ExtractedType::STuple => {
                    if let SType::STuple(_) = sf.field_tpe {
                        Ok(sf)
                    } else {
                        Err(syn::Error::new_spanned(
                            name,
                            format!(
                                "Expected tuple field of type STuple(_), got {:?}",
                                sf.field_tpe
                            ),
                        ))
                    }
                }
                crate::ergotree_proc_macro::ExtractedType::SCollection(_) => {
                    if let SType::SColl(_) = sf.field_tpe {
                        Ok(sf)
                    } else {
                        Err(syn::Error::new_spanned(
                            name,
                            format!(
                                "Expected tuple field of type SCollection(_), got {:?}",
                                sf.field_tpe
                            ),
                        ))
                    }
                }
                crate::ergotree_proc_macro::ExtractedType::SOption(_) => todo!(),
            }
        } else {
            Err(syn::Error::new_spanned(name, "Expected `typed` keyword"))
        }
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for SelectField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field_index = self.field_index;
        let input = *self.input.clone();
        tokens.extend(
            quote::quote! { ergotree_ir::mir::select_field::SelectField::new(
                #input, #field_index,
            ).unwrap()},
        )
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::convert::TryInto;

    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = SelectField::new(Expr::Const((1i64, true).into()), 1u8.try_into().unwrap())
            .unwrap()
            .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
