use crate::mir::expr::Expr;

extern crate derive_more;
use derive_more::{From, Into};

#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct Cost(u32);

#[derive(Debug)]
pub struct Costs {
    pub eq_const_size: Cost,
}

impl Costs {
    pub const DEFAULT: Costs = Costs {
        eq_const_size: Cost(1),
    };

    pub fn cost_of(&self, _: &Expr) -> Cost {
        Cost(1)
    }
}
