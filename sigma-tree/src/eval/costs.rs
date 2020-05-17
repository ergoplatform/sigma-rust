use crate::ast::Expr;

pub struct Cost(u32);

pub struct Costs {}

impl Costs {
    pub fn cost_of(&self, expr: &Expr) -> Cost {
        match expr {
            Expr::Constant { .. } => todo!(),
            Expr::Coll { .. } => todo!(),
            Expr::Tup { .. } => todo!(),
            Expr::PredefFunc(_) => todo!(),
            Expr::CollM(_) => todo!(),
            Expr::BoxM(_) => todo!(),
            Expr::CtxM(_) => todo!(),
            Expr::MethodCall { .. } => todo!(),
            Expr::BinOp(_, _, _) => todo!(),
        }
    }
}
