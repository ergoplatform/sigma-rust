use super::stype::SType;

/// Type variable for generic signatures
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct STypeVar {
    /// Type variable name (e.g. "T")
    pub name: &'static str,
}

impl STypeVar {
    /// "T" type variable
    pub const T: STypeVar = STypeVar { name: "T" };
    /// "IV"(Input Value) type variable
    pub const IV: STypeVar = STypeVar { name: "IV" };
    /// "OV"(Input Value) type variable
    pub const OV: STypeVar = STypeVar { name: "OV" };
}

/// Type parameter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeParam {
    pub(crate) ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}
