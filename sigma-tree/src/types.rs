//! Sigma types

#[derive(Clone, Debug)]
pub struct TypeCode(u8);
#[derive(PartialEq, Eq, Debug)]
pub struct MethodId(u8);
#[derive(PartialEq, Eq, Debug)]
pub struct TypeId(u8);

#[derive(PartialEq, Eq, Debug)]
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

#[derive(PartialEq, Eq, Debug)]
pub struct STypeVar {
    name: String,
}

#[derive(PartialEq, Eq, Debug)]
pub struct STypeParam {
    ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}

#[derive(PartialEq, Eq, Debug)]
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
