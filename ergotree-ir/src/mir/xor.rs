use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Byte-wise XOR op on byte arrays
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Xor {
    /// Byte array with SColl(SByte) expr type
    pub left: Box<Expr>,
    /// Byte array with SColl(SByte) expr type
    pub right: Box<Expr>,
}

impl Xor {
    /// Create new object, return error if type matching fails
    pub fn new(left: Expr, right: Expr) -> Result<Self, InvalidArgumentError> {
        let left_type : SType = left.post_eval_tpe();
        let right_type : SType = right.post_eval_tpe();
        
        match (left_type, right_type) {
            (SType::SColl(l), SType::SColl(r)) => {
                match (*l, *r) {
                    (SType::SByte, SType::SByte) => {
                        Ok(Xor {
                            left: left.into(), 
                            right: right.into()
                        })
                    }
                    (l,r) => {
                        Err(InvalidArgumentError(format!("XOR types differ: {0:?}", (l, r))))
                    }
                }
            }
            (l, r) => {
                Err(InvalidArgumentError(format!("Invalid XOR op tpe: {0:?}", (l, r)))) 
            }
        }
    }


    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(Box::new(SType::SByte))
    }
}

impl HasStaticOpCode for Xor {
    const OP_CODE: OpCode = OpCode::XOR;
}

impl SigmaSerializable for Xor {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.left.sigma_serialize(w)?;
        self.right.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let left = Expr::sigma_parse(r)?;
        let right = Expr::sigma_parse(r)?;
        Ok(Xor::new(left, right)?)
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for Xor {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(Box::new(SType::SByte)),
                depth: 0,
            }),
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(Box::new(SType::SByte)),
                depth: 0,
            }),
        )
            .prop_map(|(left, right)| Xor::new(left, right).unwrap())
            .boxed()
        }
    }
}
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<Xor>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
