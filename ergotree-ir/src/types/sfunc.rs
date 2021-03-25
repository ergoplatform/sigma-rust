use std::collections::HashMap;

use super::stype::SType;
use super::stype_param::STypeParam;
use super::stype_param::STypeVar;

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

impl SFunc {
    pub(crate) fn with_subst(self, subst: &HashMap<STypeVar, SType>) -> Self {
        let remaining_vars = self
            .tpe_params
            .into_iter()
            .filter(|v| !subst.contains_key(&v.ident))
            .collect();
        SFunc {
            t_dom: self
                .t_dom
                .iter()
                .map(|a| a.clone().with_subst(subst))
                .collect(),
            t_range: Box::new(self.t_range.with_subst(subst)),
            tpe_params: remaining_vars,
        }
    }
}
