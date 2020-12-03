use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodCall {
    pub tpe: SType,
    pub obj: Box<Expr>,
    pub method: SMethod,
    pub args: Vec<Expr>,
}

#[cfg(test)]
mod tests {
    use crate::types::scontext;

    use super::*;

    #[test]
    fn context_data_inputs() {
        let mc = MethodCall {
            tpe: scontext::DATA_INPUTS_METHOD.tpe().clone(),
            obj: Box::new(Expr::Context),
            method: scontext::DATA_INPUTS_METHOD.clone(),
            args: vec![],
        };
    }
}
