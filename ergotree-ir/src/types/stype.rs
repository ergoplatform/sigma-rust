//! SType hierarchy

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::fmt::Debug;
use hashbrown::HashMap;

use impl_trait_for_tuples::impl_for_tuples;

use crate::bigint256::BigInt256;
use crate::chain::ergo_box::ErgoBox;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::sigma_protocol::sigma_boolean::SigmaProp;
use crate::sigma_protocol::sigma_boolean::{ProveDhTuple, ProveDlog};
use crate::soft_fork::SoftForkError;
use crate::unsignedbigint256::UnsignedBigInt;
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
    /// Unsigned 256-bit integer type
    SUnsignedBigInt,
    /// ErgoBox value
    SBox,
    /// AVL tree value
    SAvlTree,
    /// Optional value
    SOption(Arc<SType>),
    /// Collection of elements of the same type
    SColl(Arc<SType>),
    /// Tuple (elements can have different types)
    STuple(STuple),
    /// Function (signature)
    SFunc(SFunc),
    /// Context object ("CONTEXT" in ErgoScript)
    SContext,
    /// UTF-8 String type
    SString,
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
            SType::SByte
                | SType::SShort
                | SType::SInt
                | SType::SLong
                | SType::SBigInt
                | SType::SUnsignedBigInt
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
                | SType::SUnit
                | SType::SGroupElement
                | SType::SSigmaProp
                | SType::SUnsignedBigInt
                | SType::SBox
                | SType::SAvlTree
                | SType::SContext
                | SType::SString
                | SType::SBoolean
                | SType::SHeader
                | SType::SPreHeader
                | SType::SGlobal
        )
    }

    pub(crate) fn check_v6_type(&self) -> Result<(), SoftForkError> {
        match self {
            SType::SUnsignedBigInt | SType::SOption(_) | SType::SHeader => {
                Err(SoftForkError::V6TypeError)
            }
            SType::SColl(elem_tpe) => elem_tpe.check_v6_type(),
            SType::STuple(tuple) => tuple.items.iter().try_for_each(SType::check_v6_type),
            _ => Ok(()),
        }
    }

    pub(crate) fn with_subst(&self, subst: &HashMap<STypeVar, SType>) -> Self {
        match self {
            SType::STypeVar(ref tpe_var) => subst.get(tpe_var).cloned().unwrap_or(self.clone()),
            SType::SOption(tpe) => SType::SOption(tpe.with_subst(subst).into()),
            SType::SColl(tpe) => SType::SColl(tpe.with_subst(subst).into()),
            SType::STuple(ref stup) => SType::STuple(stup.with_subst(subst)),
            SType::SFunc(ref sfunc) => SType::SFunc(sfunc.with_subst(subst)),
            _ => self.clone(),
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

impl core::fmt::Display for SType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SType::STypeVar(t) => write!(f, "{}", t.as_string()),
            SType::SAny => write!(f, "Any"),
            SType::SUnit => write!(f, "Unit"),
            SType::SBoolean => write!(f, "Boolean"),
            SType::SByte => write!(f, "Byte"),
            SType::SShort => write!(f, "Short"),
            SType::SInt => write!(f, "Int"),
            SType::SLong => write!(f, "Long"),
            SType::SBigInt => write!(f, "BigInt"),
            SType::SGroupElement => write!(f, "GroupElement"),
            SType::SSigmaProp => write!(f, "SigmaProp"),
            SType::SUnsignedBigInt => write!(f, "SUnsignedBigInt"),
            SType::SBox => write!(f, "Box"),
            SType::SAvlTree => write!(f, "AvlTree"),
            SType::SOption(t) => write!(f, "Option[{}]", t),
            SType::SColl(t) => write!(f, "Coll[{}]", t),
            SType::STuple(t) => write!(f, "{}", t),
            SType::SFunc(t) => write!(f, "{}", t),
            SType::SContext => write!(f, "Context"),
            SType::SString => write!(f, "String"),
            SType::SHeader => write!(f, "Header"),
            SType::SPreHeader => write!(f, "PreHeader"),
            SType::SGlobal => write!(f, "Global"),
        }
    }
}

/// Conversion to SType
pub trait LiftIntoSType {
    /// get SType
    fn stype() -> SType;
}

impl<T: LiftIntoSType> LiftIntoSType for Vec<T> {
    fn stype() -> SType {
        SType::SColl(Arc::new(T::stype()))
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

impl LiftIntoSType for UnsignedBigInt {
    fn stype() -> SType {
        SType::SUnsignedBigInt
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

impl LiftIntoSType for String {
    fn stype() -> SType {
        SType::SString
    }
}

impl<T: LiftIntoSType> LiftIntoSType for Option<T> {
    fn stype() -> SType {
        SType::SOption(Arc::new(T::stype()))
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
    use alloc::vec;
    use proptest::prelude::*;

    pub(crate) fn primitive_type(include_v6_types: bool) -> BoxedStrategy<SType> {
        let strategy = prop_oneof![
            Just(SType::SAny),
            Just(SType::SUnit),
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
            Just(SType::SString),
            Just(SType::SHeader),
            Just(SType::SPreHeader),
            Just(SType::SGlobal),
        ]
        .boxed();
        if include_v6_types {
            strategy
                .prop_union(prop_oneof![Just(SType::SUnsignedBigInt)].boxed())
                .boxed()
        } else {
            strategy
        }
    }
    impl Arbitrary for SType {
        type Parameters = bool;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![primitive_type(_args), Just(SType::STypeVar(STypeVar::t())),]
                .prop_recursive(
                    4,  // no more than this branches deep
                    64, // total elements target
                    16, // each collection max size
                    |elem| {
                        prop_oneof![
                            prop::collection::vec(elem.clone(), 2..=5)
                                .prop_map(|elems| SType::STuple(elems.try_into().unwrap())),
                            elem.clone().prop_map(|tpe| SType::SColl(Arc::new(tpe))),
                            elem.prop_map(|tpe| SType::SOption(Arc::new(tpe))),
                        ]
                    },
                )
                .boxed()
        }
    }
}
