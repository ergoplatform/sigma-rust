use crate::serialization::op_code::OpCode;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/** Represents in ErgoTree an invocation of method of the object `obj` with arguments `args`.
 * The SMethod instances in STypeCompanions may have type STypeIdent in methods types,
 * but valid ErgoTree should have SMethod instances specialized for specific types of
 * obj and args using `specializeFor`.
 * This means, if we save typeId, methodId, and we save all the arguments,
 * we can restore the specialized SMethod instance.
 * This work by induction, if we assume all arguments are monomorphic,
 * then we can make MethodCall monomorphic.
 * Thus, all ErgoTree instances are monomorphic by construction.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodCall {
    /// Object on which method will be invoked
    pub obj: Box<Expr>,
    /// Method to be invoked
    pub method: SMethod,
    /// Arguments passed to the method on invocation
    pub args: Vec<Expr>,
}

impl MethodCall {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(obj: Expr, method: SMethod, args: Vec<Expr>) -> Result<Self, InvalidArgumentError> {
        if method.tpe().t_dom.len() != args.len() + 1 {
            return Err(InvalidArgumentError(format!(
                "MethodCall: expected arguments count {} does not match provided arguments count {}",
                method.tpe().t_dom.len(), args.len() + 1)));
        }
        let mut expected_types: Vec<SType> = vec![obj.tpe()];
        let arg_types: Vec<SType> = args.clone().into_iter().map(|a| a.tpe()).collect();
        expected_types.extend(arg_types);
        if !method
            .tpe()
            .t_dom
            .iter()
            .zip(&expected_types)
            .all(|(expected, actual)| expected == actual)
        {
            return Err(InvalidArgumentError(format!(
                "MethodCall: expected types {:?} do not match provided obj and args types {:?}",
                method.tpe().t_dom,
                expected_types
            )));
        }
        Ok(Self {
            obj: obj.into(),
            method,
            args,
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        *self.method.tpe().t_range.clone()
    }
}

impl HasStaticOpCode for MethodCall {
    const OP_CODE: OpCode = OpCode::METHOD_CALL;
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for MethodCall {
    fn parse(buf: syn::parse::ParseStream) -> syn::Result<Self> {
        use crate::ergotree_proc_macro::extract_tpe_from_dot_typed;
        let _extracted_type = if buf.peek(syn::Token![.]) {
            let _dot: syn::Token![.] = buf.parse()?;
            let name: syn::Ident = buf.parse()?;
            if name != "typed" {
                return Err(syn::Error::new_spanned(
                    name.clone(),
                    format!("Expected `typed` keyword, got {}", name),
                ));
            }
            let content;
            let _bracketed = syn::bracketed!(content in buf);
            Some(extract_tpe_from_dot_typed(&content)?)
        } else {
            None
        };

        let content;
        let _paren = syn::parenthesized!(content in buf);
        let obj: Box<Expr> = content.parse()?;
        let _comma: syn::Token![,] = content.parse()?;
        let method = extract_smethod(&content)?;
        let _comma: syn::Token![,] = content.parse()?;

        // Extract method args
        let vector: syn::Ident = content.parse()?;
        if vector != "Vector" {
            return Err(syn::Error::new_spanned(
                vector.clone(),
                format!("Expected `Vector` keyword, got {}", vector),
            ));
        }
        let content_nested;
        let _paren = syn::parenthesized!(content_nested in content);
        let mut args = vec![];

        while let Ok(arg) = content_nested.parse::<Expr>() {
            args.push(arg);
            if content_nested.peek(syn::Token![,]) {
                let _comma: syn::Token![,] = content_nested.parse()?;
            }
        }
        let _comma: syn::Token![,] = content.parse()?;
        let map_ident: syn::Ident = content.parse()?;
        if map_ident != "Map" {
            return Err(syn::Error::new_spanned(
                map_ident.clone(),
                format!("Expected `Map` keyword, got {}", map_ident),
            ));
        }
        let _content_nested;
        let _paren = syn::parenthesized!(_content_nested in content);
        Ok(Self { obj, method, args })
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for MethodCall {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let obj = *self.obj.clone();
        let method = self.method.clone();
        let args = self.args.clone();
        tokens.extend(
            quote::quote! { ergotree_ir::mir::method_call::MethodCall::new(#obj, #method, vec![#(#args),*]).unwrap()}
        );
    }
}

#[cfg(feature = "ergotree-proc-macro")]
fn extract_smethod(buf: syn::parse::ParseStream) -> syn::Result<SMethod> {
    use crate::types::{scoll, sgroup_elem};

    let object_type: syn::Ident = buf.parse()?;
    match object_type.to_string().as_str() {
        "SCollection" => match extract_method_name(buf)?.as_str() {
            "flatMap" => {
                let subst = extract_concrete_types(buf)?;
                Ok(scoll::FLATMAP_METHOD.clone().with_concrete_types(&subst))
            }
            "indices" => {
                let subst = extract_concrete_types(buf)?;
                Ok(scoll::INDICES_METHOD.clone().with_concrete_types(&subst))
            }
            "zip" => {
                let subst = extract_concrete_types(buf)?;
                Ok(scoll::ZIP_METHOD.clone().with_concrete_types(&subst))
            }
            _ => todo!(),
        },
        "SGroupElement" => match extract_method_name(buf)?.as_str() {
            "getEncoded" => Ok(sgroup_elem::GET_ENCODED_METHOD.clone()),
            _ => todo!(),
        },
        _ => Err(syn::Error::new_spanned(
            object_type.clone(),
            format!("Unknown `object_type`, got {}", object_type),
        )),
    }
}

#[cfg(feature = "ergotree-proc-macro")]
fn extract_method_name(buf: syn::parse::ParseStream) -> syn::Result<String> {
    let _dot: syn::Token![.] = buf.parse()?;
    let ident: syn::Ident = buf.parse()?;
    if ident == "getMethodByName" {
        let content;
        let _paren = syn::parenthesized!(content in buf);
        let method_name: syn::LitStr = content.parse()?;
        Ok(method_name.value())
    } else {
        Err(syn::Error::new_spanned(
            ident.clone(),
            format!("Expected `getMethodByName`, got {}", ident),
        ))
    }
}

#[cfg(feature = "ergotree-proc-macro")]
fn extract_concrete_types(
    buf: syn::parse::ParseStream,
) -> syn::Result<std::collections::HashMap<crate::types::stype_param::STypeVar, SType>> {
    use crate::types::stype_param::STypeVar;

    let _dot: syn::Token![.] = buf.parse()?;
    let with_concrete_types_ident: syn::Ident = buf.parse()?;
    if with_concrete_types_ident != "withConcreteTypes" {
        return Err(syn::Error::new_spanned(
            with_concrete_types_ident.clone(),
            format!(
                "Expected `withConcreteTypes` keyword, got {}",
                with_concrete_types_ident
            ),
        ));
    }
    let content;
    let _paren = syn::parenthesized!(content in buf);
    let mut res = std::collections::HashMap::new();
    let map_ident: syn::Ident = content.parse()?;
    if map_ident == "Map" {
        let content_nested;
        let _paren = syn::parenthesized!(content_nested in content);
        loop {
            let s_type_var: syn::Ident = content_nested.parse()?;
            if s_type_var != "STypeVar" {
                return Err(syn::Error::new_spanned(
                    s_type_var.clone(),
                    format!("Expected `STypeVar` Ident, got {}", s_type_var),
                ));
            }
            let content_nested1;
            let _paren = syn::parenthesized!(content_nested1 in content_nested);
            let type_var_lit: syn::LitStr = content_nested1.parse()?;
            let type_var = match type_var_lit.value().as_str() {
                "T" => STypeVar::t(),
                "IV" => STypeVar::iv(),
                "OV" => STypeVar::ov(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        type_var_lit.clone(),
                        format!(
                            "Unknown type variable for `STypeVar`, got {:?}",
                            type_var_lit
                        ),
                    ));
                }
            };
            let _arrow: syn::Token![->] = content_nested.parse()?;
            let stype: SType = content_nested.parse()?;
            res.insert(type_var, stype);
            if !content_nested.peek(syn::Token![,]) {
                break;
            }
            let _comma: syn::Token![,] = content_nested.parse()?;
        }
        Ok(res)
    } else {
        Err(syn::Error::new_spanned(
            map_ident.clone(),
            format!("Expected `Map` Ident, got {}", map_ident),
        ))
    }
}
