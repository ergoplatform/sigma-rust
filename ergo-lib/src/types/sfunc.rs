use super::stype::SType;
use super::stype_param::STypeParam;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SFunc {
    pub t_dom: Vec<SType>,
    pub t_range: SType,
    pub tpe_params: Vec<STypeParam>,
}
