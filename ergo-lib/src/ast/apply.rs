use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Apply {
    pub func: Expr,
    pub args: Vec<Expr>,
}
