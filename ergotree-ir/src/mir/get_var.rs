//! Extracts Context variable by id and type
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// Extract value of variable from context by its ID.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GetVar {
    /// ID of variable
    pub var_id: u8,
    /// Expected type of variable
    pub var_tpe: SType,
}

impl GetVar {
    pub(crate) const OP_CODE: OpCode = OpCode::GET_VAR;

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SOption(self.var_tpe.clone().into())
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for GetVar {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.put_u8(self.var_id)?;
        self.var_tpe.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let var_id = r.get_u8()?;
        let var_tpe = SType::sigma_parse(r)?;
        Ok(Self { var_id, var_tpe })
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for GetVar {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (any::<u8>(), any::<SType>())
                .prop_map(|(var_id, var_tpe)| Self { var_id, var_tpe })
                .boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<GetVar>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
