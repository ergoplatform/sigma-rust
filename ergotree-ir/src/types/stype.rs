//! SType hierarchy

use std::collections::HashMap;
use std::convert::TryInto;

use impl_trait_for_tuples::impl_for_tuples;

use crate::bigint256::BigInt256;
use crate::chain::ergo_box::ErgoBox;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::sigma_protocol::sigma_boolean::SigmaProp;
use crate::sigma_protocol::sigma_boolean::{ProveDhTuple, ProveDlog};
use ergo_chain_types::EcPoint;

use super::sfunc::SFunc;
use super::stuple::STuple;
use super::stype_param::STypeVar;
use crate::mir::avl_tree_data::AvlTreeData;

/// Every type descriptor is a tree represented by nodes in SType hierarchy.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SType {
    /// Type variable (generic)
    STypeVar(STypeVar),
    /// TBD
    SAny,
    /// Unit struct
    SUnit,
    /// Boolean
    SBoolean,
    /// Signed byte
    SByte,
    /// Signed short (16-bit)
    SShort,
    /// Signed int (32-bit)
    SInt,
    /// Signed long (64-bit)
    SLong,
    /// 256-bit integer
    SBigInt,
    /// Discrete logarithm prime-order group element [`EcPoint`]
    SGroupElement,
    /// Proposition which can be proven and verified by sigma protocol.
    SSigmaProp,
    /// ErgoBox value
    SBox,
    /// AVL tree value
    SAvlTree,
    /// Optional value
    SOption(Box<SType>),
    /// Collection of elements of the same type
    SColl(Box<SType>),
    /// Tuple (elements can have different types)
    STuple(STuple),
    /// Function (signature)
    SFunc(SFunc),
    /// Context object ("CONTEXT" in ErgoScript)
    SContext,
    /// Header of a block
    SHeader,
    /// Header of a block without solved mining puzzle
    SPreHeader,
    /// Data type introduced to unify handling of global and non-global (i.e. methods) operations.
    SGlobal,
}

impl SType {
    /// Check if type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            SType::SByte | SType::SShort | SType::SInt | SType::SLong | SType::SBigInt
        )
    }

    /// Check if type is primitive
    pub fn is_prim(&self) -> bool {
        matches!(
            self,
            SType::SByte
                | SType::SShort
                | SType::SInt
                | SType::SLong
                | SType::SBigInt
                | SType::SAny
                | SType::SGroupElement
                | SType::SSigmaProp
                | SType::SBox
                | SType::SAvlTree
                | SType::SContext
                | SType::SBoolean
                | SType::SHeader
                | SType::SPreHeader
                | SType::SGlobal
        )
    }

    pub(crate) fn with_subst(self, subst: &HashMap<STypeVar, SType>) -> Self {
        match self {
            SType::STypeVar(ref tpe_var) => subst.get(tpe_var).cloned().unwrap_or(self),
            SType::SOption(tpe) => SType::SOption(tpe.with_subst(subst).into()),
            SType::SColl(tpe) => SType::SColl(tpe.with_subst(subst).into()),
            SType::STuple(stup) => SType::STuple(stup.with_subst(subst)),
            SType::SFunc(sfunc) => SType::SFunc(sfunc.with_subst(subst)),
            _ => self,
        }
    }
}

impl From<STuple> for SType {
    fn from(v: STuple) -> Self {
        SType::STuple(v)
    }
}

impl From<STypeVar> for SType {
    fn from(v: STypeVar) -> Self {
        SType::STypeVar(v)
    }
}

impl From<SFunc> for SType {
    fn from(v: SFunc) -> Self {
        SType::SFunc(v)
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl syn::parse::Parse for SType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        match name.to_string().as_str() {
            "SBoolean" => Ok(SType::SBoolean),
            "SAny" => Ok(SType::SAny),
            "SUnit" => Ok(SType::SUnit),
            "SByte" => Ok(SType::SByte),
            "SShort" => Ok(SType::SShort),
            "SInt" => Ok(SType::SInt),
            "SLong" => Ok(SType::SLong),
            "SBigInt" => Ok(SType::SBigInt),
            "SGroupElement" => Ok(SType::SGroupElement),
            "SSigmaProp" => Ok(SType::SSigmaProp),
            "SBox" => Ok(SType::SBox),
            "SAvlTree" => Ok(SType::SAvlTree),
            "SContext" => Ok(SType::SContext),
            "SHeader" => Ok(SType::SHeader),
            "SPreHeader" => Ok(SType::SPreHeader),
            "SGlobal" => Ok(SType::SGlobal),
            "STuple" => {
                let content;
                let _paren = syn::parenthesized!(content in input);
                Ok(SType::STuple(content.parse()?))
            }
            "SCollectionType" => {
                let content;
                let _paren = syn::parenthesized!(content in input);
                Ok(SType::SColl(content.parse()?))
            }
            "SOption" => {
                let content;
                let _paren = syn::parenthesized!(content in input);
                Ok(SType::SOption(content.parse()?))
            }
            _ => Err(syn::Error::new_spanned(
                name,
                "Unknown `SType` variant name",
            )),
        }
    }
}

#[cfg(feature = "ergotree-proc-macro")]
impl quote::ToTokens for SType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;
        tokens.extend(match self {
            SType::STypeVar(s) => quote! { ergotree_ir::types::stype::SType::STypeVar(#s) },
            SType::SAny => quote! { ergotree_ir::types::stype::SType::SAny },
            SType::SUnit => quote! { ergotree_ir::types::stype::SType::SUnit },
            SType::SBoolean => quote! { ergotree_ir::types::stype::SType::SBoolean },
            SType::SByte => quote! { ergotree_ir::types::stype::SType::SByte },
            SType::SShort => quote! { ergotree_ir::types::stype::SType::SShort },
            SType::SInt => quote! { ergotree_ir::types::stype::SType::SInt },
            SType::SLong => quote! { ergotree_ir::types::stype::SType::SLong },
            SType::SBigInt => quote! { ergotree_ir::types::stype::SType::SBigInt },
            SType::SGroupElement => quote! { ergotree_ir::types::stype::SType::SGroupElement },
            SType::SSigmaProp => quote! { ergotree_ir::types::stype::SType::SSigmaProp },
            SType::SBox => quote! { ergotree_ir::types::stype::SType::SBox },
            SType::SAvlTree => quote! { ergotree_ir::types::stype::SType::SAvlTree },
            SType::SOption(o) => {
                let tpe = *o.clone();
                quote! { ergotree_ir::types::stype::SType::SOption(Box::new(#tpe)) }
            }
            SType::SColl(c) => {
                let tpe = *c.clone();
                quote! { ergotree_ir::types::stype::SType::SColl(Box::new(#tpe)) }
            }
            SType::STuple(s) => quote! { ergotree_ir::types::stype::SType::STuple(#s) },
            SType::SFunc(f) => quote! { ergotree_ir::types::stype::SType::SFunc(#f) },
            SType::SContext => quote! { ergotree_ir::types::stype::SType::SContext },
            SType::SHeader => quote! { ergotree_ir::types::stype::SType::SHeader },
            SType::SPreHeader => quote! { ergotree_ir::types::stype::SType::SPreHeader },
            SType::SGlobal => quote! { ergotree_ir::types::stype::SType::SGlobal },
        })
    }
}

/// Conversion to SType
pub trait LiftIntoSType {
    /// get SType
    fn stype() -> SType;
}

impl<T: LiftIntoSType> LiftIntoSType for Vec<T> {
    fn stype() -> SType {
        SType::SColl(Box::new(T::stype()))
    }
}

impl LiftIntoSType for bool {
    fn stype() -> SType {
        SType::SBoolean
    }
}

impl LiftIntoSType for u8 {
    fn stype() -> SType {
        SType::SByte
    }
}

impl LiftIntoSType for i8 {
    fn stype() -> SType {
        SType::SByte
    }
}

impl LiftIntoSType for i16 {
    fn stype() -> SType {
        SType::SShort
    }
}

impl LiftIntoSType for i32 {
    fn stype() -> SType {
        SType::SInt
    }
}

impl LiftIntoSType for i64 {
    fn stype() -> SType {
        SType::SLong
    }
}

impl LiftIntoSType for ErgoBox {
    fn stype() -> SType {
        SType::SBox
    }
}

impl LiftIntoSType for SigmaBoolean {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for SigmaProofOfKnowledgeTree {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for SigmaProp {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for ProveDlog {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for EcPoint {
    fn stype() -> SType {
        SType::SGroupElement
    }
}

impl LiftIntoSType for BigInt256 {
    fn stype() -> SType {
        SType::SBigInt
    }
}

impl LiftIntoSType for ProveDhTuple {
    fn stype() -> SType {
        SType::SSigmaProp
    }
}

impl LiftIntoSType for AvlTreeData {
    fn stype() -> SType {
        SType::SAvlTree
    }
}

impl<T: LiftIntoSType> LiftIntoSType for Option<T> {
    fn stype() -> SType {
        SType::SOption(Box::new(T::stype()))
    }
}

#[impl_for_tuples(2, 4)]
#[allow(clippy::unwrap_used)]
impl LiftIntoSType for Tuple {
    fn stype() -> SType {
        let v: Vec<SType> = [for_tuples!(  #( Tuple::stype() ),* )].to_vec();
        SType::STuple(v.try_into().unwrap())
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
pub(crate) mod tests {
    use super::*;
    use proptest::prelude::*;

    pub(crate) fn primitive_type() -> BoxedStrategy<SType> {
        prop_oneof![
            Just(SType::SAny),
            Just(SType::SBoolean),
            Just(SType::SByte),
            Just(SType::SShort),
            Just(SType::SInt),
            Just(SType::SLong),
            Just(SType::SBigInt),
            Just(SType::SGroupElement),
            Just(SType::SSigmaProp),
            Just(SType::SBox),
            Just(SType::SAvlTree),
            Just(SType::SContext),
            Just(SType::SHeader),
            Just(SType::SPreHeader),
            Just(SType::SGlobal),
        ]
        .boxed()
    }

    impl Arbitrary for SType {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![primitive_type(), Just(SType::STypeVar(STypeVar::t())),]
                .prop_recursive(
                    4,  // no more than this branches deep
                    64, // total elements target
                    16, // each collection max size
                    |elem| {
                        prop_oneof![
                            prop::collection::vec(elem.clone(), 2..=5)
                                .prop_map(|elems| SType::STuple(elems.try_into().unwrap())),
                            elem.clone().prop_map(|tpe| SType::SColl(Box::new(tpe))),
                            elem.prop_map(|tpe| SType::SOption(Box::new(tpe))),
                        ]
                    },
                )
                .boxed()
        }
    }
}
