use ergo_lib::ast::expr::Expr;

#[derive(Debug, PartialEq)]
pub struct TypeCheckError {
    msg: String,
}

impl TypeCheckError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }

    pub fn pretty_desc(&self) -> String {
        self.msg.clone()
    }
}

pub fn type_check(e: Expr) -> Result<Expr, TypeCheckError> {
    // not really a relevant check, since such kind of check should be in BinOp::new()
    match &e {
        Expr::BinOp(bin) => {
            if bin.left.tpe() == bin.right.tpe() {
                Ok(e)
            } else {
                Err(TypeCheckError::new(format!(
                    "Type check error: binary op operands types do not match: {0:?}",
                    bin
                )))
            }
        }
        _ => Ok(e),
    }
}
