use core::fmt;

use crate::serialization::op_code::OpCode;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::box_methods::BoxM;
use super::coll_methods::CollM;
use super::constant::Constant;
use super::constant::ConstantPlaceholder;
use super::context_methods::ContextM;
use super::ops;
use super::predef_func::PredefFunc;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Expression in ErgoTree
pub enum Expr {
    /// Constant value
    Const(Constant),
    /// Placeholder for a constant
    ConstPlaceholder(ConstantPlaceholder),
    /// Collection of values (same type)
    Coll {
        /// Collection type
        tpe: SType,
        /// Values of the collection
        v: Vec<Expr>,
    },
    /// Tuple
    Tup {
        /// Tuple type
        tpe: SType,
        /// Values of the tuple
        v: Vec<Expr>,
    },
    /// Predefined functions (global)
    PredefFunc(PredefFunc),
    /// Collection type methods
    CollM(CollM),
    /// Box methods
    BoxM(BoxM),
    /// Context methods (i.e CONTEXT.INPUTS)
    ContextM(ContextM),
    /// Method call
    MethodCall {
        /// Method call type
        tpe: SType,
        /// Method call object
        obj: Box<Expr>,
        /// Method signature
        method: SMethod,
        /// Arguments of the method call
        args: Vec<Expr>,
    },
    /// Binary operation
    BinOp(ops::BinOp, Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Code (used in serialization)
    pub fn op_code(&self) -> OpCode {
        match self {
            Expr::Const(_) => todo!(),
            Expr::ConstPlaceholder(cp) => cp.op_code(),
            _ => todo!(),
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

impl From<Constant> for Expr {
    fn from(c: Constant) -> Self {
        Self::Const(c)
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
