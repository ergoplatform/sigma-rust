use crate::serialization::op_code::OpCode;
use crate::types::scontext::SContext;
use crate::types::stype::SType;

use super::and::And;
use super::apply::Apply;
use super::bin_op::BinOp;
use super::block::BlockValue;
use super::bool_to_sigma::BoolToSigmaProp;
use super::calc_blake2b256::CalcBlake2b256;
use super::coll_filter::Filter;
use super::coll_fold::Fold;
use super::coll_map::Map;
use super::collection::Collection;
use super::constant::Constant;
use super::constant::ConstantPlaceholder;
use super::constant::TryExtractFrom;
use super::constant::TryExtractFromError;
use super::extract_amount::ExtractAmount;
use super::extract_reg_as::ExtractRegisterAs;
use super::func_value::FuncValue;
use super::global_vars::GlobalVars;
use super::if_op::If;
use super::logical_not::LogicalNot;
use super::method_call::MethodCall;
use super::option_get::OptionGet;
use super::or::Or;
use super::property_call::PropertyCall;
use super::select_field::SelectField;
use super::upcast::Upcast;
use super::val_def::ValDef;
use super::val_use::ValUse;

extern crate derive_more;
use derive_more::From;

#[derive(PartialEq, Eq, Debug, Clone, From)]
/// Expression in ErgoTree
pub enum Expr {
    /// Constant value
    Const(Constant),
    /// Placeholder for a constant
    ConstPlaceholder(ConstantPlaceholder),
    /// Collection declaration (array of expressions of the same type)
    Collection(Collection),
    /// Predefined functions (global)
    /// Blake2b256 hash calculation
    CalcBlake2b256(CalcBlake2b256),
    Context,
    // Global(Global),
    /// Predefined global variables
    GlobalVars(GlobalVars),
    /// Function definition
    FuncValue(FuncValue),
    /// Function application
    Apply(Apply),
    /// Method call
    MethodCall(MethodCall),
    /// Property call
    ProperyCall(PropertyCall),
    /// Block (statements, followed by an expression)
    BlockValue(BlockValue),
    /// let-bound expression
    ValDef(ValDef),
    /// Reference to ValDef
    ValUse(ValUse),
    /// If, non-lazy - evaluate both branches
    If(If),
    /// Binary operation
    BinOp(BinOp),
    /// Logical AND
    And(And),
    /// Logical OR
    Or(Or),
    /// LogicalNot
    LogicalNot(LogicalNot),
    /// Option get method
    OptionGet(OptionGet),
    /// Extract register's value (box.RX properties)
    ExtractRegisterAs(ExtractRegisterAs),
    /// Collection fold op
    Fold(Fold),
    /// Collection map op
    Map(Map),
    /// Collection filter op
    Filter(Filter),
    /// Tuple field access
    SelectField(SelectField),
    /// Box monetary value
    ExtractAmount(ExtractAmount),
    /// Bool to SigmaProp
    BoolToSigmaProp(BoolToSigmaProp),
    /// Upcast numeric value
    Upcast(Upcast),
}

impl Expr {
    /// Code (used in serialization)
    pub fn op_code(&self) -> OpCode {
        match self {
            Expr::Const(_) => panic!("constant does not have op code assigned"),
            Expr::ConstPlaceholder(op) => op.op_code(),
            Expr::Collection(op) => op.op_code(),
            Expr::GlobalVars(op) => op.op_code(),
            Expr::MethodCall(op) => op.op_code(),
            Expr::ProperyCall(op) => op.op_code(),
            Expr::Context => OpCode::CONTEXT,
            Expr::OptionGet(op) => op.op_code(),
            Expr::ExtractRegisterAs(op) => op.op_code(),
            Expr::BinOp(op) => op.op_code(),
            Expr::BlockValue(op) => op.op_code(),
            Expr::ValUse(op) => op.op_code(),
            Expr::FuncValue(op) => op.op_code(),
            Expr::Apply(op) => op.op_code(),
            Expr::ValDef(op) => op.op_code(),
            Expr::ExtractAmount(op) => op.op_code(),
            Expr::SelectField(op) => op.op_code(),
            Expr::Fold(op) => op.op_code(),
            Expr::CalcBlake2b256(op) => op.op_code(),
            Expr::And(op) => op.op_code(),
            Expr::Or(op) => op.op_code(),
            Expr::LogicalNot(op) => op.op_code(),
            Expr::Map(op) => op.op_code(),
            Expr::Filter(op) => op.op_code(),
            Expr::BoolToSigmaProp(op) => op.op_code(),
            Expr::Upcast(op) => op.op_code(),
            Expr::If(op) => op.op_code(),
        }
    }

    /// Type of the expression
    pub fn tpe(&self) -> SType {
        match self {
            Expr::Const(v) => v.tpe.clone(),
            Expr::Collection(v) => v.tpe(),
            Expr::ConstPlaceholder(v) => v.tpe.clone(),
            Expr::CalcBlake2b256(v) => v.tpe(),
            Expr::Context => SType::SContext(SContext()),
            Expr::GlobalVars(v) => v.tpe(),
            Expr::FuncValue(v) => v.tpe(),
            Expr::Apply(v) => v.tpe(),
            Expr::MethodCall(v) => v.tpe(),
            Expr::ProperyCall(v) => v.tpe(),
            Expr::BlockValue(v) => v.tpe(),
            Expr::ValDef(v) => v.tpe(),
            Expr::ValUse(v) => v.tpe.clone(),
            Expr::BinOp(v) => v.tpe(),
            Expr::OptionGet(v) => v.tpe(),
            Expr::ExtractRegisterAs(v) => v.tpe.clone(),
            Expr::Fold(v) => v.tpe(),
            Expr::SelectField(v) => v.tpe(),
            Expr::ExtractAmount(v) => v.tpe(),
            Expr::And(v) => v.tpe(),
            Expr::Or(v) => v.tpe(),
            Expr::LogicalNot(v) => v.tpe(),
            Expr::Map(v) => v.tpe(),
            Expr::Filter(v) => v.tpe(),
            Expr::BoolToSigmaProp(v) => v.tpe(),
            Expr::Upcast(v) => v.tpe(),
            Expr::If(v) => v.tpe(),
        }
    }

    pub fn post_eval_tpe(&self) -> SType {
        match self.tpe() {
            SType::SFunc(sfunc) => *sfunc.t_range,
            tpe => tpe,
        }
    }

    pub fn check_post_eval_tpe(&self, expected_tpe: SType) -> Result<(), InvalidExprEvalTypeError> {
        let expr_tpe = self.post_eval_tpe();
        if expr_tpe == expected_tpe {
            Ok(())
        } else {
            Err(InvalidExprEvalTypeError(format!(
                "expected: {0:?}, got: {1:?}",
                expected_tpe, expr_tpe
            )))
        }
    }
}

/// Unexpected argument on node construction (i.e non-Option input in OptionGet)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct InvalidArgumentError(pub String);

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct InvalidExprEvalTypeError(pub String);

impl From<InvalidExprEvalTypeError> for InvalidArgumentError {
    fn from(e: InvalidExprEvalTypeError) -> Self {
        InvalidArgumentError(format!("{0:?}", e))
    }
}

impl<T: Into<Expr>> TryExtractFrom<Expr> for T {
    fn try_extract_from(v: Expr) -> Result<Self, TryExtractFromError> {
        match v {
            Expr::Const(_) => Ok(T::try_extract_from(v)?),
            Expr::ValDef(_) => Ok(T::try_extract_from(v)?),
            _ => Err(TryExtractFromError(format!(
                "Don't know how to extract {0:?} from {1:?}",
                std::any::type_name::<T>(),
                v
            ))),
        }
    }
}

#[cfg(test)]
pub mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::ast::func_value::FuncArg;
    use crate::sigma_protocol::sigma_boolean::SigmaProp;
    use crate::types::sfunc::SFunc;
    use proptest::collection::*;
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

    fn int_nested_expr(depth: usize) -> BoxedStrategy<Expr> {
        prop_oneof![any_with::<BinOp>(ArbExprParams {
            tpe: SType::SInt,
            depth
        })
        .prop_map_into(),]
        .boxed()
    }

    fn bool_nested_expr(depth: usize) -> BoxedStrategy<Expr> {
        prop_oneof![
            any_with::<BinOp>(ArbExprParams {
                tpe: SType::SBoolean,
                depth
            })
            .prop_map_into(),
            any_with::<And>(depth).prop_map_into(),
            any_with::<Or>(depth).prop_map_into(),
            any_with::<LogicalNot>(depth).prop_map_into(),
        ]
        .boxed()
    }

    fn coll_nested_expr(depth: usize, elem_tpe: &SType) -> BoxedStrategy<Expr> {
        match elem_tpe {
            SType::SBoolean => vec(bool_nested_expr(depth), 0..10)
                .prop_map(|items| Collection::new(SType::SBoolean, items).unwrap())
                .prop_map_into(),
            _ => todo!(),
        }
        .boxed()
    }

    fn any_nested_expr(depth: usize) -> BoxedStrategy<Expr> {
        prop_oneof![bool_nested_expr(depth), int_nested_expr(depth)].boxed()
    }

    fn nested_expr(tpe: SType, depth: usize) -> BoxedStrategy<Expr> {
        match tpe {
            SType::SAny => any_nested_expr(depth),
            SType::SBoolean => bool_nested_expr(depth),
            SType::SInt => int_nested_expr(depth),
            SType::SColl(elem_type) => coll_nested_expr(depth, elem_type.as_ref()),
            _ => todo!(),
        }
        .boxed()
    }

    fn int_non_nested_expr() -> BoxedStrategy<Expr> {
        prop_oneof![Just(GlobalVars::Height.into()),].boxed()
    }

    fn bool_non_nested_expr() -> BoxedStrategy<Expr> {
        prop_oneof![any_with::<Constant>(SType::SBoolean).prop_map_into()].boxed()
    }

    fn any_non_nested_expr() -> BoxedStrategy<Expr> {
        prop_oneof![int_non_nested_expr(), bool_non_nested_expr()].boxed()
    }

    fn coll_non_nested_expr(elem_tpe: &SType) -> BoxedStrategy<Expr> {
        match elem_tpe {
            SType::SByte => any_with::<Constant>(SType::SColl(Box::new(SType::SByte)))
                .prop_map(Expr::Const)
                .boxed(),
            SType::SBoolean => any_with::<Constant>(SType::SColl(Box::new(SType::SBoolean)))
                .prop_map(Expr::Const)
                .boxed(),
            _ => todo!("Collection of {0:?} is not yet implemented", elem_tpe),
        }
    }

    fn non_nested_expr(tpe: &SType) -> BoxedStrategy<Expr> {
        match tpe {
            SType::SAny => any_non_nested_expr(),
            SType::SInt => int_non_nested_expr(),
            SType::SBoolean => bool_non_nested_expr(),
            SType::SColl(elem_type) => coll_non_nested_expr(elem_type),
            _ => todo!("{0:?} is not yet implemented", tpe),
        }
    }

    fn sfunc_expr(sfunc: SFunc) -> BoxedStrategy<Expr> {
        match (sfunc.t_dom.first().unwrap(), *sfunc.t_range) {
            (SType::SBoolean, SType::SBoolean) => any_with::<Expr>(ArbExprParams {
                tpe: SType::SBoolean,
                depth: 2,
            })
            .prop_map(|expr| {
                Expr::FuncValue(FuncValue::new(
                    vec![FuncArg {
                        idx: 1.into(),
                        tpe: SType::SBoolean,
                    }],
                    expr,
                ))
            })
            .boxed(),
            _ => todo!(),
        }
    }

    impl Arbitrary for Expr {
        type Parameters = ArbExprParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            if args.depth == 0 {
                match args.tpe {
                    SType::SFunc(sfunc) => sfunc_expr(sfunc),
                    _ => prop_oneof![
                        any_with::<Constant>(args.tpe.clone())
                            .prop_map(Expr::Const)
                            .boxed(),
                        non_nested_expr(&args.tpe)
                    ]
                    .boxed(),
                }
            } else {
                nested_expr(args.tpe, args.depth - 1)
            }
        }
    }
}
