use super::stype::SType;
use super::stype_param::STypeParam;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SFunc {
    t_dom: Vec<SType>,
    t_range: SType,
    tpe_params: Vec<STypeParam>,
}
