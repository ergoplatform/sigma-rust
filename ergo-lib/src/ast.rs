//! AST for ErgoTree
use crate::eval::cost_accum::CostAccumulator;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::{serialization::op_code::OpCode, types::*};
use core::fmt;

mod constant;
pub mod ops;

pub use constant::*;

#[derive(PartialEq, Eq, Debug, Clone)]
/// newtype for box register id
pub struct RegisterId(u8);

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
    CollM(CollMethods),
    /// Box methods
    BoxM(BoxMethods),
    /// Context methods (i.e CONTEXT.INPUTS)
    CtxM(ContextMethods),
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

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Collection type instance
pub enum CollMethods {
    /// Fold method
    Fold {
        /// Collection
        input: Box<Expr>,
        /// Initial value for accumulator
        zero: Box<Expr>,
        /// Function (lambda)
        fold_op: Box<Expr>,
    },
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Box type instance
pub enum BoxMethods {
    /// Box.RX methods
    ExtractRegisterAs {
        /// Box
        input: Box<Expr>,
        /// Register id to extract value from
        register_id: RegisterId,
    },
}

impl BoxMethods {
    /// Code (serialization)
    pub fn op_code(&self) -> OpCode {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Context type instance
pub enum ContextMethods {
    /// Tx inputs
    Inputs,
    /// Tx outputs
    Outputs,
    /// Current blockchain height
    Height,
}

impl Evaluable for ContextMethods {
    fn eval(
        &self,
        _env: &crate::eval::Env,
        _ca: &mut CostAccumulator,
        ctx: &crate::eval::context::Context,
    ) -> Result<Constant, EvalError> {
        match self {
            ContextMethods::Height => Ok(ctx.height.clone()),
            _ => Err(EvalError::UnexpectedExpr),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Predefined (global) functions
pub enum PredefFunc {
    /// SHA256
    Sha256 {
        /// Byte array
        input: Box<Expr>,
    },
}
