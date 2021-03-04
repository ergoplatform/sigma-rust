use super::stype::SType;
use super::stype_param::STypeParam;

/// Function signature type
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SFunc {
    /// Function parameter types
    pub t_dom: Vec<SType>,
    /// Result type
    pub t_range: Box<SType>,
    /// Type parameters if the function is generic
    pub tpe_params: Vec<STypeParam>,
}
