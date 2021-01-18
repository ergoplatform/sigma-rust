use std::convert::TryFrom;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::types::stuple::STuple;
use crate::types::stuple::STupleItemsOutOfBoundsError;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::value::Value;

/// Tuple field access index (1..255)
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct TupleFieldIndex(u8);

impl TryFrom<u8> for TupleFieldIndex {
    type Error = STupleItemsOutOfBoundsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value >= 1 {
            Ok(TupleFieldIndex(value))
        } else {
            Err(STupleItemsOutOfBoundsError())
        }
    }
}

impl From<TupleFieldIndex> for usize {
    fn from(v: TupleFieldIndex) -> Self {
        v.0 as usize
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SelectField {
    input: Box<Expr>,
    field_index: TupleFieldIndex,
}

impl SelectField {
    pub fn new(input: Expr, field_index: TupleFieldIndex) -> Result<Self, InvalidArgumentError> {
        match input.tpe() {
            SType::STuple(STuple { items }) => {
                if items.len() >= field_index.into() {
                    Ok(SelectField {
                        input: Box::new(input),
                        field_index,
                    })
                } else {
                    Err(InvalidArgumentError(format!(
                        "SelectField field index is out of bounds {0:?}, tuple type: {1:?}",
                        field_index,
                        input.tpe()
                    )))
                }
            }
            tpe => Err(InvalidArgumentError(format!(
                "expected SelectField input type to be STuple, got {0:?}",
                tpe
            ))),
        }
    }
}

impl SelectField {
    pub fn tpe(&self) -> SType {
        match self.input.tpe() {
            SType::STuple(STuple { items }) => items.get(self.field_index).unwrap().clone(),
            tpe => panic!("expected input type to be STuple, got {0:?}", tpe),
        }
    }
}

impl Evaluable for SelectField {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::Tup(items) => items.get(self.field_index).cloned().ok_or_else(|| {
                EvalError::NotFound(format!(
                    "SelectField field index is out of bounds. Index: {0:?}, tuple: {1:?}",
                    self.field_index, items
                ))
            }),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected SelectField input to be Value::Tup, got: {0:?}",
                input_v
            ))),
        }
    }
}
