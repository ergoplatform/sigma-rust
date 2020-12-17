use std::convert::TryFrom;
use std::convert::TryInto;

use crate::chain::ergo_box::ErgoBox;
use crate::chain::ergo_box::MandatoryRegisterId;
use crate::chain::ergo_box::NonMandatoryRegisterId;
use crate::eval::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::{serialization::op_code::OpCode, types::stype::SType};

use super::constant::TryExtractInto;
use super::expr::Expr;
use super::value::Value;
use thiserror::Error;

/// newtype for box register id
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum RegisterId {
    MandatoryRegisterId(MandatoryRegisterId),
    NonMandatoryRegisterId(NonMandatoryRegisterId),
}

impl RegisterId {
    pub const R0: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R0);
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("register id {0} is out of bounds (0 - 9)")]
pub struct RegisterIdOutOfBounds(pub i8);

impl TryFrom<i8> for RegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(RegisterIdOutOfBounds(value));
        }
        let v = value as usize;
        if v < NonMandatoryRegisterId::START_INDEX {
            Ok(RegisterId::MandatoryRegisterId(value.try_into()?))
        } else if v <= NonMandatoryRegisterId::END_INDEX {
            Ok(RegisterId::NonMandatoryRegisterId(value.try_into()?))
        } else {
            Err(RegisterIdOutOfBounds(value))
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Box type instance
pub enum BoxM {
    /// Box.RX methods (get register value)
    ExtractRegisterAs {
        /// Box
        input: Box<Expr>,
        /// Register id to extract value from
        register_id: RegisterId,
        /// Type
        tpe: SType,
    },
}

impl BoxM {
    /// Code (serialization)
    pub fn op_code(&self) -> OpCode {
        todo!()
    }
}

impl Evaluable for BoxM {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        match self {
            BoxM::ExtractRegisterAs {
                input,
                register_id,
                tpe: _,
            } => Ok(Value::Opt(Box::new(
                input
                    .eval(env, ctx)?
                    .try_extract_into::<ErgoBox>()?
                    .get_register(*register_id)
                    .map(|c| c.v),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::global_vars::GlobalVars;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_box_get_reg() {
        let mc = BoxM::ExtractRegisterAs {
            input: Box::new(Expr::GlobalVars(GlobalVars::SelfBox)),
            register_id: RegisterId::MandatoryRegisterId(MandatoryRegisterId::R0),
            tpe: SType::SOption(Box::new(SType::SLong)),
        };
        let ctx = Rc::new(force_any_val::<Context>());
        let v = eval_out::<i64>(&mc.into(), ctx.clone());
        assert_eq!(v, ctx.self_box.value.as_i64());
    }
}
