use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum OptM {
    Get(Box<Expr>),
}
