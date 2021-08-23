//! XOR for collection of booleans

use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::stype::SType;

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;

/// XOR Of
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct XorOf {
    /// Collection of SBoolean
    pub input: Box<Expr>,
}

impl XorOf {
    /// Type
    pub fn tpe(&self) -> SType {
        SType::SBoolean
    }
}

impl HasStaticOpCode for XorOf {
    const OP_CODE: OpCode = OpCode::XOR_OF;
}

impl SigmaSerializable for XorOf {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        Ok(Self {
            input: Expr::sigma_parse(r)?.into(),
        })
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for XorOf {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(SType::SBoolean.into()),
                depth: args,
            })
            .prop_map(|input| Self {
                input: input.into(),
            })
            .boxed()
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any_with::<XorOf>(1)) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
