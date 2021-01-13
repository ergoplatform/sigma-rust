use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::apply::Apply;
use super::bin_op::BinOp;
use super::block::BlockValue;
use super::coll_fold::Fold;
use super::constant::Constant;
use super::constant::ConstantPlaceholder;
use super::extract_reg_as::ExtractRegisterAs;
use super::func_value::FuncValue;
use super::global_vars::GlobalVars;
use super::method_call::MethodCall;
use super::option_get::OptionGet;
use super::predef_func::PredefFunc;
use super::property_call::PropertyCall;
use super::val_def::ValDef;
use super::val_use::ValUse;

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
    Context,
    // Global(Global),
    /// Predefined global variables
    GlobalVars(Box<GlobalVars>),
    /// Function definition
    FuncValue(Box<FuncValue>),
    /// Function application
    Apply(Box<Apply>),
    /// Method call
    MethodCall(Box<MethodCall>),
    /// Property call
    ProperyCall(Box<PropertyCall>),
    /// Block (statements, followed by an expression)
    BlockValue(Box<BlockValue>),
    /// let-bound expression
    ValDef(Box<ValDef>),
    /// Reference to ValDef
    ValUse(Box<ValUse>),
    /// Binary operation
    BinOp(Box<BinOp>),
    /// Option get method
    OptionGet(Box<OptionGet>),
    /// Extract register's value (box.RX properties)
    ExtractRegisterAs(Box<ExtractRegisterAs>),
    /// Collection fold op
    Fold(Box<Fold>),
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

#[cfg(test)]
pub mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::sigma_protocol::sigma_boolean::SigmaProp;
    use proptest::prelude::*;

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct ArbExprParams {
        pub tpe: SType,
        pub depth: usize,
    }

    impl Default for ArbExprParams {
        fn default() -> Self {
            ArbExprParams {
                tpe: SType::SBoolean,
                depth: 2,
            }
        }
    }

    fn bool_nested_expr(depth: usize) -> BoxedStrategy<Expr> {
        prop_oneof![any_with::<BinOp>(ArbExprParams {
            tpe: SType::SBoolean,
            depth
        })
        .prop_map(Box::new)
        .prop_map_into()]
        .boxed()
    }

    fn any_nested_expr(depth: usize) -> BoxedStrategy<Expr> {
        prop_oneof![bool_nested_expr(depth)]
    }

    fn nested_expr(tpe: SType, depth: usize) -> BoxedStrategy<Expr> {
        match tpe {
            SType::SAny => any_nested_expr(depth),
            SType::SBoolean => bool_nested_expr(depth),
            _ => todo!(),
        }
        .boxed()
    }

    fn int_non_nested_expr() -> BoxedStrategy<Expr> {
        prop_oneof![Just(Box::new(GlobalVars::Height).into()),].boxed()
    }

    fn any_non_nested_expr() -> BoxedStrategy<Expr> {
        prop_oneof![int_non_nested_expr()]
    }

    fn non_nested_expr(tpe: &SType) -> BoxedStrategy<Expr> {
        match tpe {
            SType::SAny => any_non_nested_expr(),
            SType::SInt => int_non_nested_expr(),
            _ => todo!(),
        }
    }

    impl Arbitrary for Expr {
        type Parameters = ArbExprParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            if args.depth == 0 {
                prop_oneof![
                    any_with::<Constant>(args.tpe.clone())
                        .prop_map(Box::new)
                        .prop_map(Expr::Const)
                        .boxed(),
                    non_nested_expr(&args.tpe)
                ]
                .boxed()
            } else {
                nested_expr(args.tpe, args.depth - 1)
            }
        }
    }
}
