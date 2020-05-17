//! Sigma types

#[derive(Clone, Debug)]
pub struct TypeCode(u8);
pub struct MethodId(u8);
pub struct TypeId(u8);

#[derive(PartialEq, Eq)]
pub enum SType {
    SAny,
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
        todo!()
    }

    pub fn type_companion(&self) -> Option<STypeCompanion> {
        todo!()
    }
}

#[derive(PartialEq, Eq)]
pub struct STypeVar {
    name: String,
}

#[derive(PartialEq, Eq)]
pub struct STypeParam {
    ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}

#[derive(PartialEq, Eq)]
pub struct SFunc {
    t_dom: Vec<SType>,
    t_range: SType,
    tpe_params: Vec<STypeParam>,
}

pub struct STypeCompanion {
    pub type_id: TypeId,
    pub type_name: String,
    pub methods: Vec<SMethod>,
}

pub struct SMethod {
    pub obj_type: Box<STypeCompanion>,
    pub name: String,
    pub method_id: MethodId,
    pub tpe: SType,
}

