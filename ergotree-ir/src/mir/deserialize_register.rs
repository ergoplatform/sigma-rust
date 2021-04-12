//! Extract register of SELF box as byte array, deserialize it into Value and inline into executing script.

use super::expr::Expr;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Extract register of SELF box as byte array, deserialize it into Value and inline into executing script.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DeserializeRegister {
    /// Register number
    pub reg_n: u8,
    /// Type of value in register
    pub reg_tpe: SType,
    /// FIXME:
    pub reg_val: Option<Box<Expr>>,
}
impl DeserializeRegister {
    pub(crate) const OP_CODE: OpCode = OpCode::DESERIALIZE_REGISTER;

    /// FIXME: Is this true??
    pub fn tpe(&self) -> SType {
        self.reg_tpe.clone()
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for DeserializeRegister {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.put_u8(self.reg_n)?;
        self.reg_tpe.sigma_serialize(w)?;
        self.reg_val.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let reg_n = r.get_u8()?;
        let reg_tpe = SType::sigma_parse(r)?;
        let reg_val = Option::<Box<Expr>>::sigma_parse(r)?;
        Ok(Self {
            reg_n,
            reg_tpe,
            reg_val,
        })
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;

    use proptest::option;
    use proptest::prelude::*;

    impl Arbitrary for DeserializeRegister {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                0_u8..9,
                any::<SType>(),
                option::of(
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SColl(SType::SBoolean.into()),
                        depth: args,
                    })
                    .prop_map(Box::new),
                ),
            )
                .prop_map(|(reg_n, reg_tpe, reg_val)| Self {
                    reg_n,
                    reg_tpe,
                    reg_val,
                })
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<DeserializeRegister>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
