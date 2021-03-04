use super::stype::SType;

/// Type variable for generic signatures
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeVar {
    /// Type variable name (e.g. "T")
    pub name: &'static str,
}

impl STypeVar {
    /// "T" type variable
    pub const T: STypeVar = STypeVar { name: "T" };
}

/// Type parameter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeParam {
    ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}
