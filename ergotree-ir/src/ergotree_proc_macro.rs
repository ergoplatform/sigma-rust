//! Utility code to support `ergo_tree!` procedural-macro

use syn::parse::ParseBuffer;

use crate::types::stype::SType;

/// aa
#[derive(Debug)]
pub enum ExtractedType {
    /// Fully specified `SType`
    FullySpecified(SType),
    /// `SCollection[_]]` in scala representation.
    SCollection(Box<ExtractedType>),
    /// `SOption[_]]` in scala representation.
    SOption(Box<ExtractedType>),
    /// `STuple` in scala representation
    STuple,
}

impl From<SType> for ExtractedType {
    fn from(s: SType) -> Self {
        ExtractedType::FullySpecified(s)
    }
}

/// Extracts T within `_.typed[T]`.
/// Note that scala uses some type aliases: e.g. `BoolValue` is short for `Value[SBoolean.type]`
pub fn extract_tpe_from_dot_typed(buf: ParseBuffer) -> Result<ExtractedType, syn::Error> {
    let ident: syn::Ident = buf.parse()?;
    match &*ident.to_string() {
        "BoolValue" => Ok(SType::SBoolean.into()),
        "IntValue" => Ok(SType::SInt.into()),
        "ShortValue" => Ok(SType::SShort.into()),
        "LongValue" => Ok(SType::SLong.into()),
        "BigIntValue" => Ok(SType::SBigInt.into()),
        "ByteValue" => Ok(SType::SByte.into()),
        "SigmaPropValue" => Ok(SType::SSigmaProp.into()),
        "Value" => {
            let content;
            let _bracketed = syn::bracketed!(content in buf);
            let next_ident: syn::Ident = content.parse()?;
            match &*next_ident.to_string() {
                "STuple" => Ok(ExtractedType::STuple),
                "SByte" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SByte))
                }
                "SGroupElement" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SGroupElement))
                }
                "SInt" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SInt))
                }
                "SLong" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SLong))
                }
                "SBigInt" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SBigInt))
                }
                "SBoolean" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SBoolean))
                }
                "SAvlTree" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SAvlTree))
                }
                "SBox" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SBox))
                }
                "SSigmaProp" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SSigmaProp))
                }
                "SHeader" => {
                    handle_dot_type(buf)?;
                    Ok(ExtractedType::FullySpecified(SType::SHeader))
                }
                "SOption" => {
                    let content;
                    let _bracketed = syn::bracketed!(content in buf);
                    Ok(ExtractedType::SOption(Box::new(
                        extract_tpe_from_dot_typed(content)?,
                    )))
                }
                "SCollection" => {
                    let content;
                    let _bracketed = syn::bracketed!(content in buf);
                    Ok(ExtractedType::SCollection(Box::new(
                        extract_tpe_from_dot_typed(content)?,
                    )))
                }
                _ => {
                    unreachable!("unknown ident T in _.typed[Value[T]]")
                }
            }
        }
        _ => unreachable!("unknown ident T in _.typed[T]"),
    }
}

/// Parses `.type` from the buffered token stream
pub fn handle_dot_type(buf: ParseBuffer) -> Result<ParseBuffer, syn::Error> {
    let _dot: syn::Token![.] = buf.parse()?;
    let ident: syn::Ident = buf.parse()?;
    if ident != "type" {
        return Err(syn::Error::new_spanned(ident, ""));
    }
    Ok(buf)
}
