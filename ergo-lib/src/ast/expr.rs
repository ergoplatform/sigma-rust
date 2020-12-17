use core::fmt;

use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::box_methods::BoxM;
use super::coll_methods::CollM;
use super::constant::Constant;
use super::constant::ConstantPlaceholder;
use super::global_vars::GlobalVars;
use super::method_call::MethodCall;
use super::ops;
use super::option_get::OptionGet;
use super::predef_func::PredefFunc;
use super::property_call::PropertyCall;

extern crate derive_more;
use derive_more::From;

#[derive(PartialEq, Eq, Debug, Clone, From)]
/// Expression in ErgoTree
pub enum Expr {
    /// Constant value
    Const(Constant),
    /// Placeholder for a constant
    ConstPlaceholder(ConstantPlaceholder),
    /// Predefined functions (global)
    PredefFunc(PredefFunc),
    /// Collection type methods
    CollM(CollM),
    /// Box methods
    BoxM(BoxM),
    Context,
    // Global(Global),
    /// Predefined global variables
    GlobalVars(GlobalVars),
    /// Method call
    MethodCall(MethodCall),
    /// Property call
    ProperyCall(PropertyCall),
    /// Binary operation
    BinOp(ops::BinOp, Box<Expr>, Box<Expr>),
    /// Option get method
    OptionGet(Box<OptionGet>),
}

impl Expr {
    /// Code (used in serialization)
    pub fn op_code(&self) -> OpCode {
        match self {
            Expr::Const(_) => todo!(),
            Expr::ConstPlaceholder(cp) => cp.op_code(),
            Expr::GlobalVars(v) => v.op_code(),
            Expr::MethodCall(v) => v.op_code(),
            Expr::ProperyCall(v) => v.op_code(),
            Expr::Context => OpCode::CONTEXT,
            _ => todo!("{0:?}", self),
        }
    }

    /// Type of the expression
    pub fn tpe(&self) -> &SType {
        match self {
            Expr::Const(c) => &c.tpe,
            _ => todo!(),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::sigma_protocol::sigma_boolean::SigmaProp;
    use proptest::prelude::*;

    impl Arbitrary for Expr {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![any::<Constant>().prop_map(Expr::Const)].boxed()
        }
    }
}
