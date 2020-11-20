//! SType hierarchy

use crate::chain::ergo_box::ErgoBox;
use crate::serialization::types::TypeCode;
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::sigma_protocol::sigma_boolean::SigmaProp;

use super::sfunc::SFunc;
use super::stype_companion::STypeCompanion;

/// Every type descriptor is a tree represented by nodes in SType hierarchy.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SType {
    /// TBD
    SAny,
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
    STup(Vec<SType>),
    /// Function (signature)
    SFunc(Box<SFunc>),
}

impl SType {
    /// Type code used in serialization of SType values.
    pub fn type_code(&self) -> TypeCode {
        match self {
            SType::SAny => todo!(),
            SType::SBoolean => TypeCode::SBOOLEAN,
            SType::SByte => TypeCode::SBYTE,
            SType::SShort => TypeCode::SSHORT,
            SType::SInt => TypeCode::SINT,
            SType::SLong => TypeCode::SLONG,
            SType::SBigInt => TypeCode::SBIGINT,
            SType::SGroupElement => TypeCode::SGROUP_ELEMENT,
            SType::SSigmaProp => TypeCode::SSIGMAPROP,
            SType::SBox => todo!(),
            SType::SAvlTree => todo!(),
            SType::SOption(_) => todo!(),
            SType::SColl(_) => todo!(),
            SType::STup(_) => todo!(),
            SType::SFunc(_) => todo!(),
        }
    }

    /// Get STypeCompanion instance associated with this SType
    pub fn type_companion(&self) -> Option<STypeCompanion> {
        todo!()
    }

    /// Create new SColl with the given element type
    pub fn new_scoll(elem_type: SType) -> SType {
        SType::SColl(Box::new(elem_type))
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn primitive_type() -> BoxedStrategy<SType> {
        prop_oneof![
            Just(SType::SBoolean),
            Just(SType::SByte),
            Just(SType::SShort),
            Just(SType::SInt),
            Just(SType::SLong),
            Just(SType::SBigInt),
            Just(SType::SGroupElement),
            Just(SType::SSigmaProp),
        ]
        .boxed()
    }

    impl Arbitrary for SType {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                primitive_type(),
                primitive_type().prop_map(SType::new_scoll),
            ]
            .boxed()
        }
    }
}
