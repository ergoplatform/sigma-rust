use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/// If (lazy evaluation)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct If {
    /// Condition (SBoolean)
    pub condition: Box<Expr>,
    /// Expr, evaluated if condition is True
    pub true_branch: Box<Expr>,
    /// Expr evaluated if condition is False
    pub false_branch: Box<Expr>,
}

impl If {
    /// Type
    pub fn tpe(&self) -> SType {
        self.true_branch.tpe()
    }
}

impl HasStaticOpCode for If {
    const OP_CODE: OpCode = OpCode::IF;
}

impl SigmaSerializable for If {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.condition.sigma_serialize(w)?;
        self.true_branch.sigma_serialize(w)?;
        self.false_branch.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let condition = Expr::sigma_parse(r)?.into();
        let true_branch = Expr::sigma_parse(r)?.into();
        let false_branch = Expr::sigma_parse(r)?.into();
        Ok(Self {
            condition,
            true_branch,
            false_branch,
        })
    }
}

#[cfg(feature = "arbitrary")]
/// Arbitrary impl
mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for If {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SBoolean,
                    depth: 2,
                }),
                any::<Expr>(),
            )
                .prop_map(|(condition, true_branch)| Self {
                    condition: condition.into(),
                    true_branch: true_branch.clone().into(),
                    false_branch: true_branch.into(),
                })
                .boxed()
        }
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

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<If>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
