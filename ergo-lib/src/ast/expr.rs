use core::fmt;

use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::bin_op::BinOp;
use super::coll_fold::Fold;
use super::constant::Constant;
use super::constant::ConstantPlaceholder;
use super::extract_reg_as::ExtractRegisterAs;
use super::global_vars::GlobalVars;
use super::method_call::MethodCall;
use super::option_get::OptionGet;
use super::predef_func::PredefFunc;
use super::property_call::PropertyCall;

extern crate derive_more;
use derive_more::From;

#[derive(PartialEq, Eq, Debug, Clone, From)]
/// Expression in ErgoTree
pub enum Expr {
    /// Constant value
    Const(Box<Constant>),
    /// Placeholder for a constant
    ConstPlaceholder(Box<ConstantPlaceholder>),
    /// Predefined functions (global)
    PredefFunc(Box<PredefFunc>),
    /// Collection fold op
    Fold(Box<Fold>),
    Context,
    // Global(Global),
    /// Predefined global variables
    GlobalVars(Box<GlobalVars>),
    /// Method call
    MethodCall(Box<MethodCall>),
    /// Property call
    ProperyCall(Box<PropertyCall>),
    /// Binary operation
    BinOp(Box<BinOp>),
    /// Option get method
    OptionGet(Box<OptionGet>),
    /// Extract register's value (box.RX properties)
    ExtractRegisterAs(Box<ExtractRegisterAs>),
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
            Expr::OptionGet(v) => v.op_code(),
            Expr::ExtractRegisterAs(v) => v.op_code(),
            Expr::BinOp(op) => op.op_code(),
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
pub mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::sigma_protocol::sigma_boolean::SigmaProp;
    use proptest::prelude::*;

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct ArbExprParams {
        pub tpe: SType,
        pub nesting_level: usize,
    }

    impl Default for ArbExprParams {
        fn default() -> Self {
            ArbExprParams {
                tpe: SType::SBoolean,
                nesting_level: 4,
            }
        }
    }

    fn arb_bool_expr(nesting_level: usize) -> BoxedStrategy<Expr> {
        prop_oneof![any_with::<BinOp>(ArbExprParams {
            tpe: SType::SBoolean,
            nesting_level
        })
        .prop_map(Box::new)
        .prop_map_into()]
        .boxed()
    }

    fn any_expr(nesting_level: usize) -> BoxedStrategy<Expr> {
        prop_oneof![arb_bool_expr(nesting_level)]
    }

    fn arb_expr_with_type(tpe: SType, nesting_level: usize) -> BoxedStrategy<Expr> {
        match tpe {
            SType::SAny => any_expr(nesting_level),
            SType::SBoolean => arb_bool_expr(nesting_level),
            // SType::SByte => {}
            // SType::SShort => {}
            // SType::SInt => {}
            // SType::SLong => {}
            // SType::SBigInt => {}
            // SType::SGroupElement => {}
            // SType::SSigmaProp => {}
            // SType::SBox => {}
            // SType::SAvlTree => {}
            // SType::SOption(_) => {}
            // SType::SColl(_) => {}
            // SType::STuple(_) => {}
            // SType::SFunc(_) => {}
            // SType::SContext(_) => {}
            _ => todo!(),
        }
        .boxed()
    }

    impl Arbitrary for Expr {
        type Parameters = ArbExprParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            if args.nesting_level == 0 {
                any_with::<Constant>(args.tpe)
                    .prop_map(Box::new)
                    .prop_map(Expr::Const)
                    .boxed()
            } else {
                arb_expr_with_type(args.tpe, args.nesting_level - 1)
            }
        }
    }
}
