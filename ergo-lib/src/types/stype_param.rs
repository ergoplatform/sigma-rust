use super::stype::SType;

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
