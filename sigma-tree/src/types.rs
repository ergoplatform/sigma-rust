//! Sigma types

use super::serialization::types::TypeCode;

#[derive(PartialEq, Eq, Debug)]
pub struct MethodId(u8);
#[derive(PartialEq, Eq, Debug)]
pub struct TypeId(u8);

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SType {
    SAny,
    SBoolean,
    SByte,
    SShort,
    SInt,
    SLong,
    SBigInt,
    SGroupElement,
    SSigmaProp,
    SBox,
    SAvlTree,
    SOption(Box<SType>),
    SColl(Box<SType>),
    STup(Vec<SType>),
    SFunc(Box<SFunc>),
}

impl SType {
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

    pub fn type_companion(&self) -> Option<STypeCompanion> {
        todo!()
    }

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

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeVar {
    name: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeParam {
    ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SFunc {
    t_dom: Vec<SType>,
    t_range: SType,
    tpe_params: Vec<STypeParam>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct STypeCompanion {
    pub type_id: TypeId,
    pub type_name: String,
    pub methods: Vec<SMethod>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct SMethod {
    pub obj_type: Box<STypeCompanion>,
    pub name: String,
    pub method_id: MethodId,
    pub tpe: SType,
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
