use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Fold {
    /// Collection
    input: Expr,
    /// Initial value for accumulator
    zero: Expr,
    /// Function (lambda)
    fold_op: Expr,
}

impl SigmaSerializable for Fold {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.zero.sigma_serialize(w)?;
        self.fold_op.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let zero = Expr::sigma_parse(r)?;
        let fold_op = Expr::sigma_parse(r)?;
        Ok(Fold {
            input,
            zero,
            fold_op,
        })
    }
}
