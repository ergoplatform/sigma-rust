use ergo_lib::ast::expr::Expr;

#[derive(Debug, PartialEq)]
pub struct TypeCheckError {}

pub fn type_check(e: Expr) -> Result<Expr, TypeCheckError> {
    // not really a relevant check, since such kind of check should be in BinOp::new()
    match &e {
        Expr::BinOp(bin) => {
            if bin.left.tpe() == bin.right.tpe() {
                Ok(e)
            } else {
                Err(TypeCheckError {})
            }
        }
        _ => Ok(e),
    }
}
