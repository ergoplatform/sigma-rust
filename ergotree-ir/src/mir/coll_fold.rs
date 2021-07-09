use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stuple::STuple;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/// Applies a binary function to a start value and all elements of this collection,
/// going left to right.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Fold {
    /// Collection
    pub input: Box<Expr>,
    /// Initial value for accumulator
    pub zero: Box<Expr>,
    /// Function (lambda)
    pub fold_op: Box<Expr>,
}

impl Fold {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, zero: Expr, fold_op: Expr) -> Result<Self, InvalidArgumentError> {
        let input_elem_type: SType = match input.post_eval_tpe() {
            SType::SColl(elem_type) => Ok(*elem_type),
            _ => Err(InvalidArgumentError(format!(
                "Expected Fold input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }?;
        match fold_op.tpe() {
            SType::SFunc(sfunc)
                if sfunc.t_dom == vec![STuple::pair(zero.tpe(), input_elem_type).into()] =>
            {
                Ok(Fold {
                    input: input.into(),
                    zero: zero.into(),
                    fold_op: fold_op.into(),
                })
            }
            _ => Err(InvalidArgumentError(format!(
                "Invalid fold_op tpe: {0:?}",
                fold_op.tpe()
            ))),
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        self.zero.tpe()
    }
}

impl HasStaticOpCode for Fold {
    const OP_CODE: OpCode = OpCode::FOLD;
}

impl SigmaSerializable for Fold {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.zero.sigma_serialize(w)?;
        self.fold_op.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?.into();
        let zero = Expr::sigma_parse(r)?.into();
        let fold_op = Expr::sigma_parse(r)?.into();
        Ok(Fold {
            input,
            zero,
            fold_op,
        })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    impl Arbitrary for Fold {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), any::<Expr>(), any::<Expr>())
                .prop_map(|(input, zero, fold_op)| Self {
                    input: input.into(),
                    zero: zero.into(),
                    fold_op: fold_op.into(),
                })
                .boxed()
        }
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<Fold>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
