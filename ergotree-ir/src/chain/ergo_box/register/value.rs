use derive_more::From;
use thiserror::Error;

use crate::mir::constant::Constant;
use crate::mir::constant::Literal;
use crate::mir::expr::Expr;
use crate::mir::tuple::Tuple;
use crate::serialization::SigmaSerializable;

/// Register value (either Constant or bytes if it's unparseable)
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub enum RegisterValue {
    /// Constant value
    Parsed(Constant),
    /// Parsed evaluated Tuple expression
    /// see https://github.com/ergoplatform/sigma-rust/issues/700
    ParsedTupleExpr(EvaluatedTuple),
    /// Unparseable bytes
    Invalid {
        /// Bytes that were not parsed (whole register bytes)
        bytes: Vec<u8>,
        /// Error message on parsing
        error_msg: String,
    },
}

/// Ensures that tuple only contains Constant values
/// see https://github.com/ergoplatform/sigma-rust/issues/700
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub struct EvaluatedTuple {
    tuple: Tuple,
    constant: Constant,
}

impl EvaluatedTuple {
    /// Create new EvaluatedTuple from Tuple, returns error if it contains non-Constant values
    pub fn new(tuple: Tuple) -> Result<EvaluatedTuple, RegisterValueError> {
        match tuple_to_constant(&tuple) {
            Ok(constant) => Ok(EvaluatedTuple { tuple, constant }),
            Err(e) => Err(RegisterValueError::InvalidTupleExpr(format!(
                "tuple expr {tuple:?} contain non-constant items: {e}"
            ))),
        }
    }

    /// Get inner Tuple
    pub fn to_tuple_expr(&self) -> Expr {
        self.tuple.clone().into()
    }

    /// Convert to Constant
    pub fn as_constant(&self) -> &Constant {
        &self.constant
    }
}

/// Errors on parsing register values
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RegisterValueError {
    /// Invalid register value
    #[error("Invalid register value: {0}")]
    Invalid(String),
    /// Invalid Tuple expression in the parsed regiser value
    #[error("Invalid Tuple expression in the parsed regiser value: {0}")]
    InvalidTupleExpr(String),
    /// Unexpected register value
    #[error("Unexpected register value: {0}")]
    UnexpectedRegisterValue(String),
}

impl RegisterValue {
    /// Return a Constant if it's parsed, otherwise None
    pub fn as_constant(&self) -> Result<&Constant, RegisterValueError> {
        match self {
            RegisterValue::Parsed(c) => Ok(c),
            RegisterValue::ParsedTupleExpr(t) => Ok(t.as_constant()),
            RegisterValue::Invalid {
                bytes: _,
                error_msg,
            } => Err(RegisterValueError::Invalid(error_msg.to_string())),
        }
    }

    /// Return a seraialized bytes of the register value
    #[allow(clippy::unwrap_used)] // it could only fail on OOM, etc.
    pub fn sigma_serialize_bytes(&self) -> Vec<u8> {
        match self {
            RegisterValue::Parsed(c) => c.sigma_serialize_bytes().unwrap(),
            RegisterValue::ParsedTupleExpr(t) => t.to_tuple_expr().sigma_serialize_bytes().unwrap(),
            RegisterValue::Invalid {
                bytes,
                error_msg: _,
            } => bytes.clone(),
        }
    }

    /// Parse bytes to RegisterValue
    pub fn sigma_parse_bytes(bytes: &[u8]) -> Self {
        if let Ok(expr) = Expr::sigma_parse_bytes(bytes) {
            match expr {
                Expr::Const(c) => RegisterValue::Parsed(c),
                Expr::Tuple(t) => match EvaluatedTuple::new(t) {
                    Ok(et) => RegisterValue::ParsedTupleExpr(et),
                    Err(e) => RegisterValue::Invalid {
                        bytes: bytes.to_vec(),
                        error_msg: format!("invalid tuple in register value : {e:?}"),
                    },
                },
                e => RegisterValue::Invalid {
                    bytes: bytes.to_vec(),
                    error_msg: format!(
                        "Unexpected parsed register value: {e:?} from bytes {0:?}",
                        bytes
                    ),
                },
            }
        } else {
            RegisterValue::Invalid {
                bytes: bytes.to_vec(),
                error_msg: format!("failed to parse register value: {0:?}", bytes),
            }
        }
    }
}

/// Convert evaluated Tuple expression to Constant
/// see https://github.com/ergoplatform/sigma-rust/issues/700
fn tuple_to_constant(t: &Tuple) -> Result<Constant, String> {
    let values = t.items.try_mapped_ref(|tuple_item| match tuple_item {
        Expr::Const(c) => Ok(c.v.clone()),
        Expr::Tuple(t) => Ok(tuple_to_constant(t)?.v),
        e => return Err(format!("Unexpected value in tuple: {e:?}")),
    })?;
    let v = Literal::Tup(values);
    let c = Constant { tpe: t.tpe(), v };
    Ok(c)
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::types::stuple::STuple;
    use crate::types::stype::SType;

    use super::*;

    #[test]
    fn test_tuple_expr_i700() {
        let tuple_expr_bytes_str = "860202660263";
        let tuple_expr_bytes = base16::decode(tuple_expr_bytes_str).unwrap();
        assert!(
            Constant::sigma_parse_bytes(&tuple_expr_bytes).is_err(),
            "constant cannot be parsed from tuple expr"
        );
        let reg_value = RegisterValue::sigma_parse_bytes(&tuple_expr_bytes);
        // now let's construct a Constant for (102, 99) byte tuple
        let expected_constant: Constant = Constant {
            tpe: SType::STuple(STuple::pair(SType::SByte, SType::SByte)),
            v: Literal::Tup([Literal::Byte(102), Literal::Byte(99)].into()),
        };
        assert_eq!(
            reg_value.as_constant().unwrap(),
            &expected_constant,
            "should be accessible as Constant"
        );
        assert_eq!(
            reg_value.sigma_serialize_bytes(),
            tuple_expr_bytes,
            "preserve tuple expr on serialization"
        );
    }
}
