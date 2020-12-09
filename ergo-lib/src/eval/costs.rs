use crate::ast::expr::Expr;

extern crate derive_more;
use derive_more::{From, Into};

#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct Cost(u32);

pub struct Costs {}

impl Costs {
    pub const DEFAULT: Costs = Costs {};
}

impl Costs {
    pub fn cost_of(&self, _: &Expr) -> Cost {
        Cost(1)
    }
}
