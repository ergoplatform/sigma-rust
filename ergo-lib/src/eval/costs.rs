use crate::ast::Expr;

pub struct Cost(u32);

pub struct Costs {}

impl Costs {
    pub const DEFAULT: Costs = Costs {};
}

impl Costs {
    pub fn cost_of(&self, _: &Expr) -> Cost {
        todo!()
    }
}
