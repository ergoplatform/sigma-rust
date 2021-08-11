//! Extract register of SELF box as `Coll[Byte]`, deserialize it into Value and inline into executing script.

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::stype::SType;

/// Extract register of SELF box as `Coll[Byte]`, deserialize and execute Expr out of it
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DeserializeRegister {
    /// Register number (0 .. =9 for R0-R9 registers)
    pub reg: u8,
    /// Type of expression serialized in register
    pub tpe: SType,
    /// Default value (expression that would be executed if register is empty)
    pub default: Option<Box<Expr>>,
}
impl DeserializeRegister {
    /// Type
    pub fn tpe(&self) -> SType {
        self.tpe.clone()
    }
}

impl HasStaticOpCode for DeserializeRegister {
    const OP_CODE: OpCode = OpCode::DESERIALIZE_REGISTER;
}

impl SigmaSerializable for DeserializeRegister {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.reg)?;
        self.tpe.sigma_serialize(w)?;
        self.default.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let reg = r.get_u8()?;
        let tpe = SType::sigma_parse(r)?;
        let default = Option::<Box<Expr>>::sigma_parse(r)?;
        Ok(Self { reg, tpe, default })
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
                .prop_map(|(reg, tpe, default)| Self { reg, tpe, default })
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
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
