//! Traits for IR nodes with one input value(expr)

use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// IR nodes with one input value(expr)
pub trait UnaryOp {
    /// Input value(expr) of the IR node
    fn input(&self) -> &Expr;
}

/// Constructor for unary IR nodes that check the validity of the argument
pub trait UnaryOpTryBuild: Sized {
    /// Create new IR node, returns an error if any of the requirements failed
    fn try_build(input: Expr) -> Result<Self, InvalidArgumentError>;
}

impl<T: UnaryOp + UnaryOpTryBuild> SigmaSerializable for T {
    fn sigma_serialize<W: SigmaByteWrite>(
        &self,
        w: &mut W,
    ) -> crate::serialization::SigmaSerializeResult {
        self.input().sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let input = Expr::sigma_parse(r)?;
        let r = T::try_build(input)?;
        Ok(r)
    }
}
