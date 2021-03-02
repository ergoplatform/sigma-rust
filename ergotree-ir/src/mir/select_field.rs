use std::convert::TryFrom;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
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

impl SigmaSerializable for TupleFieldIndex {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.put_u8(self.0)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let field_index = r.get_u8()?;
        TupleFieldIndex::try_from(field_index).map_err(|_| {
            SerializationError::ValueOutOfBounds(format!(
                "invalid tuple field index: {0}",
                field_index
            ))
        })
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

    pub fn op_code(&self) -> OpCode {
        OpCode::SELECT_FIELD
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

impl SigmaSerializable for SelectField {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.field_index.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?.into();
        let field_index = TupleFieldIndex::sigma_parse(r)?;
        Ok(SelectField { input, field_index })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::convert::TryInto;

    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = SelectField::new(Expr::Const((1i64, true).into()), 1u8.try_into().unwrap())
            .unwrap()
            .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}
