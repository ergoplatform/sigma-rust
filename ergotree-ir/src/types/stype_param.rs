use super::stype::SType;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeVar {
    pub name: &'static str,
}

impl STypeVar {
    pub const T: STypeVar = STypeVar { name: "T" };
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeParam {
    ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}
