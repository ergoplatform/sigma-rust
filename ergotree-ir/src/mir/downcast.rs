//! Numerical downcast

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::stype::SType;

use crate::has_opcode::HasStaticOpCode;

/// Numerical downcast
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Downcast {
    /// Numerical value to be downcast
    pub input: Box<Expr>,
    /// Target type for the input value to be downcast to
    pub tpe: SType,
}

impl Downcast {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(input: Expr, target_tpe: SType) -> Result<Self, InvalidArgumentError> {
        if !target_tpe.is_numeric() {
            return Err(InvalidArgumentError(format!(
                "Downcast: expected target type to be numeric, got {:?}",
                target_tpe
            )));
        }
        let post_eval_tpe = input.post_eval_tpe();
        if post_eval_tpe.is_numeric() {
            Ok(Self {
                input: input.into(),
                tpe: target_tpe,
            })
        } else {
            Err(InvalidArgumentError(format!(
                "Downcast: expected input value type to be numeric, got {:?}",
                post_eval_tpe
            )))
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        self.tpe.clone()
    }
}

impl HasStaticOpCode for Downcast {
    const OP_CODE: OpCode = OpCode::DOWNCAST;
}

impl SigmaSerializable for Downcast {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.input.sigma_serialize(w)?;
        self.tpe.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let input = Expr::sigma_parse(r)?.into();
        let tpe = SType::sigma_parse(r)?;
        Ok(Downcast { input, tpe })
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for Downcast {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SLong,
                depth: 2,
            })
            .prop_map(|input| Downcast::new(input, SType::SInt).unwrap())
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
pub mod proptests {

    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Downcast>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
