//! AND conjunction for sigma propositions

use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// AND conjunction for sigma propositions
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SigmaAnd {
    /// Collection of SSigmaProp
    pub items: Vec<Expr>,
}

impl SigmaAnd {
    pub(crate) const OP_CODE: OpCode = OpCode::SIGMA_AND;

    /// Create new object, returns an error if any of the requirements failed
    pub fn new(items: Vec<Expr>) -> Result<Self, InvalidArgumentError> {
        let item_types: Vec<SType> = items
            .clone()
            .into_iter()
            .map(|it| it.post_eval_tpe())
            .collect();
        if item_types
            .iter()
            .all(|tpe| matches!(tpe, SType::SSigmaProp))
        {
            Ok(Self { items })
        } else {
            Err(InvalidArgumentError(format!(
                "Sigma conjecture: expected all items be of type SSigmaProp, got {:?},\n items: {:?}",
                item_types, items
            )))
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SSigmaProp
    }

    pub(crate) fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for SigmaAnd {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.items.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self::new(Vec::<Expr>::sigma_parse(r)?)?)
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use crate::mir::constant::Constant;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for SigmaAnd {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any_with::<Constant>(SType::SSigmaProp.into()), 0..5)
                .prop_map(|constants| Self {
                    items: constants.into_iter().map(|c| c.into()).collect(),
                })
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
        fn ser_roundtrip(v in any::<SigmaAnd>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}
